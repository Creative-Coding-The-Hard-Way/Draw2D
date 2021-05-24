use draw2d::graphics::{
    ext::Texture2dFactory,
    vulkan::{buffer::CpuBuffer, texture::TextureImage, Device},
};

use ab_glyph::{Font, Glyph, Point, ScaleFont};
use anyhow::Result;
use ash::vk;
use std::{marker::PhantomData, sync::Arc};

pub struct TextRenderer<F: Font, SF: ScaleFont<F>> {
    font: SF,
    _p: PhantomData<F>,
}

pub fn standard_glyphs() -> String {
    r###"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890~!@#$%^&*()[]{}/\";:.<>"###.into()
}

impl<F: Font, SF: ScaleFont<F>> TextRenderer<F, SF> {
    /// create a new text renderer for a particular font.
    pub fn new(font: SF) -> Self {
        Self {
            font,
            _p: PhantomData,
        }
    }

    pub fn build_atlas_texture(
        &self,
        contents: String,
        device: &Arc<Device>,
    ) -> Result<TextureImage> {
        let (glyphs, atlas_bounds) =
            layout_padded_glyphs(&self.font, 4.0, &contents, 1024.0);

        let (width, height) = (
            atlas_bounds.width() as usize,
            atlas_bounds.height() as usize,
        );
        let mut glyph_bytes = vec![0u8; width * height * 4];

        glyphs.into_iter().for_each(|glyph| {
            let offset = glyph.position;
            let outlined_glyph = self.font.outline_glyph(glyph).unwrap();
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
    let mut caret = position + ab_glyph::point(0.0, font.ascent());
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
        caret.x += font.h_advance(glyph.id);

        glyphs.push(glyph);
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

        glyphs.push(glyph);
    }

    (glyphs, bounds)
}
