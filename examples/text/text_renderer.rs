use draw2d::{
    geometry::Rect,
    graphics::{
        ext::Texture2dFactory,
        layer::Batch,
        texture_atlas::{TextureAtlas, TextureHandle},
        vertex::Vertex2d,
        vulkan::{buffer::CpuBuffer, texture::TextureImage, Device},
        Graphics,
    },
};

use ab_glyph::{Font, Glyph, GlyphId, Point, ScaleFont};
use anyhow::Result;
use ash::vk;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub struct TextRenderer<F: Font, SF: ScaleFont<F>> {
    font: SF,
    texture_handle: TextureHandle,
    glyph_tex_coords: HashMap<GlyphId, Rect<f32>>,
    _p: PhantomData<F>,
}

impl<F: Font, SF: ScaleFont<F>> TextRenderer<F, SF> {
    /// create a new text renderer for a particular font.
    pub fn new(font: SF, graphics: &mut Graphics) -> Result<Self> {
        let (texture, glyph_tex_coords) =
            build_glyph_atlas(&font, &graphics.device)?;

        let texture_handle = graphics.add_texture(texture)?;

        Ok(Self {
            font,
            texture_handle,
            glyph_tex_coords,
            _p: PhantomData,
        })
    }

    /// Layout the entire set of renderable glyphs for the current font.
    ///
    /// # Params
    ///
    /// - line_length: the character length of each line
    /// - pos: the location for the baseline of the rendered text
    pub fn layout_debug(
        &self,
        line_length: usize,
        pos: [f32; 2],
        color: [f32; 4],
    ) -> Batch {
        let full_text = self
            .font
            .codepoint_ids()
            .enumerate()
            .flat_map(|(i, (_glyph_id, c))| {
                if i != 0 && i % line_length == 0 {
                    Some('\n')
                } else {
                    None
                }
                .into_iter()
                .chain(std::iter::once(c))
            })
            .collect::<String>();

        self.layout_text(&full_text, pos, color)
    }

    /// Render text with baseline at the given location.
    ///
    /// Multiple batches from this renderer can be merged into a single
    /// render batch if desired.
    pub fn layout_text(
        &self,
        text: &str,
        pos: [f32; 2],
        color: [f32; 4],
    ) -> Batch {
        let glyphs =
            layout_paragraph(&self.font, ab_glyph::point(pos[0], pos[1]), text);

        let mut batch = Batch::default();
        batch.texture_handle = self.texture_handle;

        for glyph in glyphs {
            self.triangulate_glyph(glyph, color, &mut batch.vertices);
        }

        batch
    }

    /// Destroy the texture in the graphics subsystem's texture atlas.
    ///
    /// # Unsafe Because
    ///
    /// - the atlas will not successfully render text after this call, the
    ///   application is responsible for disposing of any remaining batches
    pub unsafe fn destroy_texture(
        &mut self,
        graphics: &mut Graphics,
    ) -> Result<()> {
        graphics.take_texture(self.texture_handle)?;
        Ok(())
    }

    fn triangulate_glyph(
        &self,
        glyph: Glyph,
        rgba: [f32; 4],
        vertices: &mut Vec<Vertex2d>,
    ) {
        let rect_option = self.glyph_tex_coords.get(&glyph.id);
        if rect_option.is_none() {
            return;
        }

        let rect = rect_option.unwrap();
        let outlined = self.font.outline_glyph(glyph).unwrap();
        let bounds = outlined.px_bounds();

        Quad {
            top_left: Vertex2d {
                pos: [bounds.min.x, bounds.min.y],
                uv: [rect.left, rect.top],
                rgba,
            },
            top_right: Vertex2d {
                pos: [bounds.max.x, bounds.min.y],
                uv: [rect.right, rect.top],
                rgba,
            },
            bottom_right: Vertex2d {
                pos: [bounds.max.x, bounds.max.y],
                uv: [rect.right, rect.bottom],
                rgba,
            },
            bottom_left: Vertex2d {
                pos: [bounds.min.x, bounds.max.y],
                uv: [rect.left, rect.bottom],
                rgba,
            },
        }
        .triangulate(vertices)
    }
}

/// Simple paragraph layout for glyphs into `target`.
/// Account for `\n` newlines and kerning between glyphs.
///
/// # Params
///
/// - font: the scaled font to use for selecting and aligning glyphs
/// - position: the starting position for the line of text
/// - text: the text to compute render into glyphs
///
fn layout_paragraph<F, SF>(font: &SF, position: Point, text: &str) -> Vec<Glyph>
where
    F: Font,
    SF: ScaleFont<F>,
{
    let mut glyphs = vec![];
    glyphs.reserve(text.len());

    let v_advance = font.height() + font.line_gap();
    let mut caret = position + ab_glyph::point(0.0, font.ascent().ceil());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = ab_glyph::point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id).ceil();

        if !c.is_whitespace() {
            glyphs.push(glyph);
        }
    }

    glyphs
}

fn build_glyph_atlas<F, SF>(
    font: &SF,
    device: &Arc<Device>,
) -> Result<(TextureImage, HashMap<GlyphId, Rect<f32>>)>
where
    F: Font,
    SF: ScaleFont<F>,
{
    let (glyphs, atlas_bounds) = layout_padded_glyphs(font, 4.0);

    let (width, height) = (
        atlas_bounds.width() as usize,
        atlas_bounds.height() as usize,
    );
    let mut glyph_bytes = vec![0u8; width * height * 4];
    let mut glyph_tex_coords = HashMap::new();

    glyphs.into_iter().for_each(|glyph| {
        let offset = glyph.position;
        let id = glyph.id;
        let outlined_glyph = font.outline_glyph(glyph).unwrap();
        let bounds = outlined_glyph.px_bounds();

        glyph_tex_coords.insert(
            id,
            Rect {
                left: offset.x / atlas_bounds.width(),
                right: (offset.x + bounds.width()) / atlas_bounds.width(),
                top: offset.y / atlas_bounds.height(),
                bottom: (offset.y + bounds.height()) / atlas_bounds.height(),
            },
        );

        outlined_glyph.draw(|x, y, v| {
            let x_o = x + offset.x as u32;
            let y_o = y + offset.y as u32;
            let index = (x_o + y_o * width as u32) as usize * 4;
            glyph_bytes[index + 0] = 255;
            glyph_bytes[index + 1] = 255;
            glyph_bytes[index + 2] = 255;
            glyph_bytes[index + 3] = (v * 255.0) as u8;
        });
    });

    let mut texture = device.create_empty_2d_texture(
        "Font Atlas",
        width as u32,
        height as u32,
        1,
    )?;

    unsafe {
        let mut transfer_buffer =
            CpuBuffer::new(device.clone(), vk::BufferUsageFlags::TRANSFER_SRC)?;
        transfer_buffer.write_data(&glyph_bytes)?;
        texture.upload_from_buffer(&transfer_buffer)?;
    }

    Ok((texture, glyph_tex_coords))
}

/// Position every glyph in the font such that each can be rendered without
/// any overlap and with a bit of padding between each glyph.
fn layout_padded_glyphs<F, SF>(
    font: &SF,
    padding: f32,
) -> (Vec<Glyph>, Rect<f32>)
where
    F: Font,
    SF: ScaleFont<F>,
{
    let target_width = font.scale().y * 32.0;

    let mut bounds = Rect::<f32> {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    };

    let v_advance = font.height() + padding;

    let mut glyphs = vec![];
    glyphs.reserve(font.glyph_count());

    let mut caret = ab_glyph::point(padding, padding);
    for (_glyph_id, c) in font.codepoint_ids() {
        if c.is_control() {
            continue;
        }

        // assign the glyph's position, ensure that it is always exactly
        // pixel-aligned.
        let mut glyph = font.scaled_glyph(c);
        caret.x = caret.x.ceil();
        caret.y = caret.y.ceil();
        glyph.position = caret;

        let outline_option = font.outline_glyph(glyph.clone());
        if outline_option.is_none() {
            continue;
        }
        let outline = outline_option.unwrap();

        let glyph_bounds = outline.px_bounds();

        caret.x += glyph_bounds.width() + padding;
        bounds.right = bounds.right.max(caret.x);

        if caret.x >= target_width {
            caret.y += v_advance;
            caret.x = padding;
        }

        bounds.bottom =
            bounds.bottom.max(caret.y + glyph_bounds.height() + padding);

        glyphs.push(glyph);
    }

    (glyphs, bounds)
}

struct Quad {
    top_left: Vertex2d,
    top_right: Vertex2d,
    bottom_left: Vertex2d,
    bottom_right: Vertex2d,
}

impl Quad {
    pub fn triangulate(&self, vertices: &mut Vec<Vertex2d>) {
        vertices.extend_from_slice(&[
            // upper triangle
            self.top_left,
            self.top_right,
            self.bottom_right,
            // lower triangle
            self.top_left,
            self.bottom_right,
            self.bottom_left,
        ]);
    }
}
