// Copyright (C) 2022, Alex Badics
//
// This file is part of Hun-Law.
//
// Hun-law is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Hun-law is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hun-law. If not, see <http://www.gnu.org/licenses/>.

use anyhow::{bail, Result};
use log::debug;

use super::{
    font::{FastFont, ToUnicodeResult},
    textstate::TextState,
    util::fix_character_coding_quirks,
    CropBox,
};

const DEFAULT_WIDTH_OF_SPACE: f32 = 0.25;

#[derive(Debug)]
pub struct PositionedChar {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub width_of_space: f32,
    pub bold: bool,
    pub content: char,
}

#[derive(Debug, Default)]
pub struct CharCollector {
    pub chars: Vec<PositionedChar>,
    width_of_space: f32,
    current_font_is_bold: bool,
    crop: CropBox,
}

impl CharCollector {
    pub fn new(crop: CropBox) -> Self {
        Self {
            crop,
            ..Default::default()
        }
    }

    fn render_character(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        scaling: f32,
        c: char,
    ) -> Result<()> {
        let width_of_space = self.width_of_space * scaling;
        let content = fix_character_coding_quirks(c);
        if !content.is_whitespace() && self.crop.is_inside(x, y) {
            self.chars.push(PositionedChar {
                x,
                y,
                width,
                width_of_space,
                content,
                bold: self.current_font_is_bold,
            });
        }
        Ok(())
    }

    pub fn render_multiple_characters(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        scaling: f32,
        chars: &str,
    ) -> Result<()> {
        // Horrible hack to separate ligatures into graphemes.
        // We don't really need to be exact, this 'x' information will probably not be used
        let x_step = width / (chars.chars().count() as f32);
        for (i, c) in chars.chars().enumerate() {
            self.render_character(x + i as f32 * x_step, y, x_step, scaling, c)?
        }
        Ok(())
    }

    pub fn render_cid(&mut self, state: &mut TextState, font: &FastFont, cid: u32) -> Result<()> {
        let rendering_matrix = state.rendering_matrix();
        let w0 = font.widths.get(cid as usize) / 1000.0;
        match font.to_unicode(cid) {
            ToUnicodeResult::Unknown => bail!("Unknown CID: {}", cid),
            ToUnicodeResult::Char(c) => self.render_character(
                rendering_matrix.m31,
                rendering_matrix.m32,
                w0 * rendering_matrix.m11,
                rendering_matrix.m11,
                c,
            )?,
            ToUnicodeResult::String(s) => self.render_multiple_characters(
                rendering_matrix.m31,
                rendering_matrix.m32,
                w0 * rendering_matrix.m11,
                rendering_matrix.m11,
                s,
            )?,
        }
        state.advance_by_char(cid);
        Ok(())
    }

    pub fn font_changed(&mut self, font: &FastFont) -> Result<()> {
        self.width_of_space = font.widths.get(32) / 1000.;
        if self.width_of_space == 0.0 || self.width_of_space == 1.0 {
            debug!("Had to use default space width for font");
            self.width_of_space = DEFAULT_WIDTH_OF_SPACE;
        };
        self.current_font_is_bold = font
            .name
            .as_ref()
            .map_or(false, |n| n.contains("bold") || n.contains("Bold"));
        Ok(())
    }
}
