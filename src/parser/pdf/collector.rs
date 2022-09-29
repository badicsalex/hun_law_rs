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

use crate::util::indentedline::{IndentedLine, IndentedLinePart, EMPTY_LINE};

use super::{
    font::{FastFont, ToUnicodeResult},
    state::State,
    util::{compare_float_for_sorting, fix_character_coding_quirks},
    CropBox, PageOfLines,
};

const SAME_LINE_EPSILON: f32 = 0.5;
const ADDITIONAL_EMPTY_LINE_THRESHOLD: f32 = 16.0;
const DEFAULT_WIDTH_OF_SPACE: f32 = 0.25;
const SPACE_DETECTION_THRESHOLD_RATIO: f32 = 0.5;
const JUSTIFIED_DETECTION_THRESHOLD_RATIO: f32 = 0.8;

#[derive(Debug)]
struct PositionedChar {
    x: f32,
    y: f32,
    width: f32,
    width_of_space: f32,
    bold: bool,
    content: char,
}

#[derive(Debug, Default)]
pub struct CharCollector {
    chars: Vec<PositionedChar>,
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

    fn consolidate_line(
        mut chars: Vec<PositionedChar>,
        estimated_right_margin: f32,
    ) -> IndentedLine {
        chars.sort_unstable_by(|c1, c2| compare_float_for_sorting(c1.x, c2.x));
        let last_char = match chars.last() {
            Some(x) => x,
            None => return EMPTY_LINE,
        };
        let justified = last_char.x
            + last_char.width
            + last_char.width_of_space * JUSTIFIED_DETECTION_THRESHOLD_RATIO
            >= estimated_right_margin;
        let mut result = Vec::<IndentedLinePart>::new();
        let mut threshold_to_space = None;
        let mut prev_x = 0.0;
        for current_char in &chars {
            if let Some(threshold_to_space) = threshold_to_space {
                // The exception for '„' is needed, because
                // Visually, there is very little space between the starting quote and the
                // text before it, but logically, there should always be a space character.
                if current_char.x > threshold_to_space || current_char.content == '„' {
                    result.push(IndentedLinePart {
                        dx: (threshold_to_space - prev_x) as f64,
                        content: ' ',
                        bold: current_char.bold,
                    });
                    prev_x = threshold_to_space;
                }
            }
            result.push(IndentedLinePart {
                dx: (current_char.x - prev_x) as f64,
                content: current_char.content,
                bold: current_char.bold,
            });
            prev_x = current_char.x;
            threshold_to_space = Some(
                current_char.x
                    + current_char.width
                    + current_char.width_of_space * SPACE_DETECTION_THRESHOLD_RATIO,
            );
        }
        while let Some(IndentedLinePart { content: ' ', .. }) = result.get(0) {
            result.remove(0);
        }
        while let Some(IndentedLinePart { content: ' ', .. }) = result.last() {
            result.pop();
        }
        IndentedLine::from_parts(result, justified)
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

    pub fn render_cid(&mut self, state: &mut State, font: &FastFont, cid: u32) -> Result<()> {
        let rendering_matrix = state.rendering_matrix();
        let w0 = font.widths.get(cid as usize) / 1000.0;
        let spacing = state.char_spacing + if cid == 32 { state.word_spacing } else { 0.0 };
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
        let tx = (w0 * state.font_size + spacing) * state.horizontal_scale;
        state.advance(tx);
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

impl TryFrom<CharCollector> for PageOfLines {
    type Error = anyhow::Error;

    fn try_from(value: CharCollector) -> Result<Self, Self::Error> {
        let estimated_right_margin = value
            .chars
            .iter()
            .fold(0.0_f32, |acc, c| acc.max(c.x + c.width));
        let mut result = Vec::<IndentedLine>::new();
        let mut chars = value.chars;
        chars.sort_unstable_by(|c1, c2| compare_float_for_sorting(c2.y, c1.y));
        let mut current_line = Vec::<PositionedChar>::new();
        for current_char in chars {
            let y_diff = if current_line.is_empty() {
                0.0
            } else {
                (current_line[0].y - current_char.y).abs()
            };
            // TODO: instad of 0.2, use some real line height thing
            // 0.2 is small enough not to trigger for the e.g. the 2 in "m2" (the unit).
            // And this is okay for now
            if y_diff < SAME_LINE_EPSILON {
                current_line.push(current_char);
            } else {
                result.push(CharCollector::consolidate_line(
                    current_line,
                    estimated_right_margin,
                ));
                // Add empty line on a "big-enough gap"
                // Should be based on actual font height, but this is
                // good enough for the rest of the parsing steps.
                if y_diff > ADDITIONAL_EMPTY_LINE_THRESHOLD {
                    result.push(EMPTY_LINE);
                }
                current_line = vec![current_char];
            }
        }
        result.push(CharCollector::consolidate_line(
            current_line,
            estimated_right_margin,
        ));
        Ok(PageOfLines { lines: result })
    }
}
