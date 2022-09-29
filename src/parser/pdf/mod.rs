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

mod collector;
mod font;
mod state;
mod util;

use anyhow::{anyhow, Result};
use euclid::Transform2D;
use pdf::object::{PageRc, Resolve};
use serde::Serialize;

use self::{collector::CharCollector, font::FontCache, state::State};
use crate::util::indentedline::IndentedLine;

/// Box in PDF coorinates
///
/// Coordinates start from bottom left. Unit is 'point', there are 72 'points' in an inch
/// An A4 page is 595 x 842 points
/// A typical margin is 0.75 inches.
#[derive(Debug, Clone)]
pub struct CropBox {
    pub bottom: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
}

impl CropBox {
    fn is_inside(&self, x: f32, y: f32) -> bool {
        self.left <= x && self.right >= x && self.bottom <= y && self.top >= y
    }
}
impl Default for CropBox {
    fn default() -> Self {
        Self {
            bottom: 0.0,
            left: 0.0,
            top: 1000.0,
            right: 1000.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PageOfLines {
    pub lines: Vec<IndentedLine>,
}

fn render_page(
    page: PageRc,
    pdf_file: &impl Resolve,
    font_cache: &mut FontCache,
    collector: &mut CharCollector,
) -> Result<()> {
    let contents = page.contents.as_ref().unwrap();
    let resources = page.resources()?;
    let mut state = State::default();
    let mut state_stack = Vec::new();
    for op in contents.operations(pdf_file)? {
        match op {
            pdf::content::Op::BeginMarkedContent { .. } => (),
            pdf::content::Op::EndMarkedContent => (),
            pdf::content::Op::MarkedContentPoint { .. } => (),
            pdf::content::Op::Save => state_stack.push(state.clone()),
            pdf::content::Op::Restore => {
                state = state_stack
                    .pop()
                    .ok_or_else(|| anyhow!("State stack empty"))?
            }
            pdf::content::Op::Transform { matrix } => {
                state.graphics_matrix = state.graphics_matrix.then(&matrix.into())
            }
            pdf::content::Op::GraphicsState { name } => {
                let new_state = resources
                    .graphics_states
                    .get(&name)
                    .ok_or_else(|| anyhow!("Did not find graphics state resource"))?;
                if let Some((font_ref, size)) = new_state.font {
                    let font = pdf_file.get(font_ref)?;
                    let fast_font = font_cache.get(&*font, pdf_file)?;
                    collector.font_changed(&fast_font)?;
                    state.font = Some(fast_font);
                    state.font_size = size;
                }
            }
            pdf::content::Op::BeginText => {
                state.text_matrix = Transform2D::identity();
                state.line_matrix = Transform2D::identity();
            }
            pdf::content::Op::EndText => (),
            pdf::content::Op::CharSpacing { char_space } => state.char_spacing = char_space,
            pdf::content::Op::WordSpacing { word_space } => state.word_spacing = word_space,
            pdf::content::Op::TextScaling { horiz_scale } => state.horizontal_scale = horiz_scale,
            pdf::content::Op::Leading { leading } => state.leading = leading,
            pdf::content::Op::TextFont { name, size } => {
                let font = resources
                    .fonts
                    .get(&name)
                    .ok_or_else(|| anyhow!("Did not find font resource"))?;
                let fast_font = font_cache.get(&*font, pdf_file)?;
                collector.font_changed(&fast_font)?;
                state.font = Some(fast_font);
                state.font_size = size;
            }
            pdf::content::Op::TextRise { rise } => state.rise = rise,
            pdf::content::Op::MoveTextPosition { translation } => {
                state.set_both_matrices(
                    Transform2D::translation(translation.x, translation.y).then(&state.line_matrix),
                );
            }
            pdf::content::Op::SetTextMatrix { matrix } => {
                state.set_both_matrices(matrix.into());
            }
            pdf::content::Op::TextNewline => {
                state.set_both_matrices(
                    Transform2D::translation(0.0, -state.leading).then(&state.line_matrix),
                );
            }
            pdf::content::Op::TextDraw { text } => {
                collector.render_text(&mut state, text)?;
            }
            pdf::content::Op::TextDrawAdjusted { array } => {
                for item in array {
                    match item {
                        pdf::content::TextDrawAdjusted::Text(text) => {
                            collector.render_text(&mut state, text)?
                        }
                        pdf::content::TextDrawAdjusted::Spacing(delta) => {
                            state.advance(
                                -delta * state.horizontal_scale * state.font_size / 1000.0,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn parse_pdf(buffer: &[u8], crop: CropBox) -> Result<Vec<PageOfLines>> {
    let pdf_file = pdf::file::File::from_data(buffer)?;
    let mut font_cache = FontCache::default();
    let mut result = Vec::new();
    for page in pdf_file.pages() {
        let page = page?;
        let mut collector = CharCollector::new(crop.clone());
        render_page(page, &pdf_file, &mut font_cache, &mut collector)?;
        result.push(collector.try_into()?);
    }
    Ok(result)
}
