use draw2d::graphics::{
    ext::Texture2dFactory,
    vertex::Vertex2d,
    vulkan::{buffer::CpuBuffer, texture::TextureImage, Device},
};

use ab_glyph::{Font, Glyph, GlyphId, Point, ScaleFont};
use anyhow::Result;
use ash::vk;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

#[derive(Debug, Copy, Clone)]
struct Rect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub struct TextRenderer<F: Font, SF: ScaleFont<F>> {
    font: SF,
    glyph_tex_coords: HashMap<GlyphId, Rect>,
    _p: PhantomData<F>,
}

pub fn standard_glyphs() -> String {
    r###"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890~!@#$%^&*()[]{}/\"';:.<>-_"###.into()
}

impl<F: Font, SF: ScaleFont<F>> TextRenderer<F, SF> {
    /// create a new text renderer for a particular font.
    pub fn new(font: SF) -> Self {
        Self {
            font,
            glyph_tex_coords: HashMap::new(),
            _p: PhantomData,
        }
    }

    pub fn single_letter(&self, c: char, pos: [f32; 2]) -> Vec<Vertex2d> {
        let mut glyph = self.font.scaled_glyph(c);
        glyph.position = ab_glyph::point(pos[0], pos[1]);

        let rect = self.glyph_tex_coords.get(&glyph.id).unwrap();
        let outlined = self.font.outline_glyph(glyph).unwrap();
        let bounds = outlined.px_bounds();

        Quad {
            top_left: Vertex2d {
                pos: [bounds.min.x, bounds.min.y],
                uv: [rect.left, rect.top],
                ..Default::default()
            },
            top_right: Vertex2d {
                pos: [bounds.max.x, bounds.min.y],
                uv: [rect.right, rect.top],
                ..Default::default()
            },
            bottom_right: Vertex2d {
                pos: [bounds.max.x, bounds.max.y],
                uv: [rect.right, rect.bottom],
                ..Default::default()
            },
            bottom_left: Vertex2d {
                pos: [bounds.min.x, bounds.max.y],
                uv: [rect.left, rect.bottom],
                ..Default::default()
            },
        }
        .triangulate()
    }

    pub fn layout_Text(&self, text: &str, pos: [f32; 2]) -> Vec<Vertex2d> {
        let glyphs =
            layout_paragraph(&self.font, ab_glyph::point(pos[0], pos[1]), text);

        let vertices = glyphs
            .into_iter()
            .map(|glyph| {
                let rect = self.glyph_tex_coords.get(&glyph.id).unwrap();
                let outlined = self.font.outline_glyph(glyph).unwrap();
                let bounds = outlined.px_bounds();
                Quad {
                    top_left: Vertex2d {
                        pos: [bounds.min.x, bounds.min.y],
                        uv: [rect.left, rect.top],
                        ..Default::default()
                    },
                    top_right: Vertex2d {
                        pos: [bounds.max.x, bounds.min.y],
                        uv: [rect.right, rect.top],
                        ..Default::default()
                    },
                    bottom_right: Vertex2d {
                        pos: [bounds.max.x, bounds.max.y],
                        uv: [rect.right, rect.bottom],
                        ..Default::default()
                    },
                    bottom_left: Vertex2d {
                        pos: [bounds.min.x, bounds.max.y],
                        uv: [rect.left, rect.bottom],
                        ..Default::default()
                    },
                }
                .triangulate()
            })
            .flatten()
            .collect::<Vec<Vertex2d>>();

        vertices
    }

    pub fn build_atlas_texture(
        &mut self,
        contents: String,
        device: &Arc<Device>,
    ) -> Result<TextureImage> {
        let (glyphs, atlas_bounds) =
            layout_padded_glyphs(&self.font, 4.0, &contents, 512.0);

        let (width, height) = (
            atlas_bounds.width() as usize,
            atlas_bounds.height() as usize,
        );
        let mut glyph_bytes = vec![0u8; width * height * 4];

        glyphs.into_iter().for_each(|glyph| {
            let offset = glyph.position;
            let id = glyph.id;
            let outlined_glyph = self.font.outline_glyph(glyph).unwrap();
            let bounds = outlined_glyph.px_bounds();

            self.glyph_tex_coords.insert(
                id,
                Rect {
                    left: offset.x / atlas_bounds.width(),
                    right: (offset.x + bounds.width()) / atlas_bounds.width(),
                    top: offset.y / atlas_bounds.height(),
                    bottom: (offset.y + bounds.height())
                        / atlas_bounds.height(),
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
            let mut transfer_buffer = CpuBuffer::new(
                device.clone(),
                vk::BufferUsageFlags::TRANSFER_SRC,
            )?;
            transfer_buffer.write_data(&glyph_bytes)?;
            texture.upload_from_buffer(&transfer_buffer)?;
        }

        Ok(texture)
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

/// Layout a string of glyphs with extra padding between each.
///
/// Kerning and control characters are ignored.
///
/// # Params
///
/// - font: the scaled font to use for selecting and aligning glyphs
/// - position: the starting position for the line of text
/// - text: the text to compute render into glyphs
///
fn layout_padded_glyphs<F, SF>(
    font: &SF,
    padding: f32,
    text: &str,
    max_width: f32,
) -> (Vec<Glyph>, ab_glyph::Rect)
where
    F: Font,
    SF: ScaleFont<F>,
{
    let mut bounds = ab_glyph::Rect {
        min: ab_glyph::point(0.0, 0.0),
        max: ab_glyph::point(0.0, 0.0),
    };

    let v_advance = font.height() + padding;

    let mut glyphs = vec![];
    glyphs.reserve(text.len());

    let mut caret = ab_glyph::point(padding, padding);
    for c in text.chars() {
        caret.x = caret.x.ceil();
        caret.y = caret.y.ceil();

        if c.is_control() {
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        glyph.position = caret;

        let outline = font.outline_glyph(glyph.clone()).unwrap();

        let glyph_bounds = outline.px_bounds();

        caret.x += glyph_bounds.width() + padding;

        bounds.max.x = bounds.max.x.max(caret.x);

        if caret.x >= max_width {
            caret.y += v_advance;
            caret.x = padding;
        }

        bounds.max.y =
            bounds.max.y.max(caret.y + glyph_bounds.height() + padding);

        if !c.is_whitespace() {
            glyphs.push(glyph);
        }
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
    pub fn triangulate(&self) -> Vec<Vertex2d> {
        vec![
            // upper triangle
            self.top_left,
            self.top_right,
            self.bottom_right,
            // lower triangle
            self.top_left,
            self.bottom_right,
            self.bottom_left,
        ]
    }
}
