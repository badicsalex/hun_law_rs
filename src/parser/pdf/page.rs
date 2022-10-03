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

use std::rc::Rc;

use anyhow::{anyhow, Result};
use euclid::Transform2D;
use pdf::{
    content::{Op, TextDrawAdjusted},
    font::Font,
    object::{MaybeRef, PageRc, Resolve, Resources, XObject},
    primitive::{Name, PdfString},
};

use super::{
    collector::CharCollector,
    font::{FastFont, FontCache},
    textstate::TextState,
};
use crate::parser::pdf::actual_text::ActualTextCollector;

#[derive(Debug)]
pub struct PageRenderer<'a, TR>
where
    TR: Resolve,
{
    page: PageRc,
    pdf_file: &'a TR,
    font_cache: &'a mut FontCache,
    collector: &'a mut CharCollector,
    resources: Option<MaybeRef<Resources>>,

    state: TextState,
    state_stack: Vec<TextState>,

    marked_content_stack: Vec<MarkedContentType>,

    actual_text_collector: Option<ActualTextCollector>,
}

#[derive(Debug, PartialEq, Eq)]
enum MarkedContentType {
    ActualText,
    Other,
}

impl<'a, TR> PageRenderer<'a, TR>
where
    TR: Resolve,
{
    pub fn new(
        page: PageRc,
        pdf_file: &'a TR,
        font_cache: &'a mut FontCache,
        collector: &'a mut CharCollector,
    ) -> Result<Self> {
        Ok(Self {
            resources: page.resources.clone(),
            page,
            pdf_file,
            font_cache,
            collector,
            state: TextState::default(),
            state_stack: Vec::new(),
            marked_content_stack: Vec::new(),
            actual_text_collector: None,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.page
            .contents
            .as_ref()
            .ok_or_else(|| anyhow!("No contents in PDF file"))?
            .operations(self.pdf_file)?
            .into_iter()
            .try_for_each(|op| self.handle_op(op))
    }

    fn font(&self) -> Result<Rc<FastFont>> {
        self.state
            .font
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Trying to draw text without a font"))
    }

    fn resources(&self) -> Result<MaybeRef<Resources>> {
        self.resources
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("No resources in PDF"))
    }

    fn render_cid(&mut self, cid: u32) -> Result<()> {
        let font = self.font()?;
        if let Some(actual_text) = &mut self.actual_text_collector {
            actual_text.render_cid(&mut self.state, &font, cid)
        } else {
            self.collector.render_cid(&mut self.state, &font, cid)
        }
    }

    fn render_text(&mut self, text: PdfString) -> Result<()> {
        let data = text.as_bytes();
        if self.font()?.is_cid {
            data.chunks_exact(2).try_for_each(|s| -> Result<()> {
                let cid = u16::from_be_bytes(s.try_into()?);
                self.render_cid(cid as u32)?;
                Ok(())
            })?;
        } else {
            data.iter()
                .try_for_each(|cid| self.render_cid(*cid as u32))?;
        }

        Ok(())
    }

    fn set_font(&mut self, font: &Font, size: f32) -> Result<()> {
        let fast_font = self.font_cache.get(font, self.pdf_file)?;
        self.collector.font_changed(&fast_font)?;
        self.state.font = Some(fast_font);
        self.state.font_size = size;
        Ok(())
    }

    fn handle_xobject(&mut self, name: Name) -> Result<()> {
        let resources = self.resources()?;
        let xobject_ref = resources
            .xobjects
            .get(&name)
            .ok_or_else(|| anyhow!("Xobject not found in PDF: {}", name))?;
        let xobject = self.pdf_file.get(*xobject_ref)?;
        let xfo = if let XObject::Form(xfo) = &*xobject {
            xfo
        } else {
            // TODO: Maybe the other types will be interesting
            return Ok(());
        };

        // Save some stuff on this function's stack
        let saved_resources = self.resources.clone();
        if let Some(resources) = xfo.dict().resources.clone() {
            self.resources = Some(resources);
        }
        self.state_stack.push(self.state.clone());

        // Recurse
        for op in xfo.operations(self.pdf_file)? {
            self.handle_op(op)?
        }

        // Load back the stuff
        self.state = self
            .state_stack
            .pop()
            .ok_or_else(|| anyhow!("State stack empty after Form rendering"))?;
        self.resources = saved_resources;
        Ok(())
    }

    fn handle_op(&mut self, op: Op) -> Result<()> {
        match op {
            // --- Marked content ---
            Op::BeginMarkedContent { tag, properties } => {
                let mct = if let Some(atc) = ActualTextCollector::from_bmc_params(tag, properties) {
                    self.actual_text_collector = Some(atc);
                    MarkedContentType::ActualText
                } else {
                    MarkedContentType::Other
                };
                self.marked_content_stack.push(mct);
            }
            Op::EndMarkedContent => {
                let mct = self
                    .marked_content_stack
                    .pop()
                    .ok_or_else(|| anyhow!("Marked content stack was empty when popped"))?;
                if mct == MarkedContentType::ActualText {
                    if let Some(actual_text) = &self.actual_text_collector {
                        actual_text.finish(self.collector)?;
                    }
                    self.actual_text_collector = None;
                }
            }

            // --- State stack ---
            Op::Save => self.state_stack.push(self.state.clone()),
            Op::Restore => {
                self.state = self
                    .state_stack
                    .pop()
                    .ok_or_else(|| anyhow!("State stack empty"))?
            }

            // --- Font stuff ---
            Op::GraphicsState { name } => {
                if let Some((font_ref, size)) = self
                    .resources()?
                    .graphics_states
                    .get(&name)
                    .ok_or_else(|| anyhow!("Did not find graphics state resource"))?
                    .font
                {
                    self.set_font(&*self.pdf_file.get(font_ref)?, size)?;
                }
            }
            Op::TextFont { name, size } => {
                let resources = self.resources()?;
                let font = resources
                    .fonts
                    .get(&name)
                    .ok_or_else(|| anyhow!("Did not find font resource"))?;
                self.set_font(font, size)?;
            }

            // --- Positioning ---
            Op::BeginText => {
                self.state.text_matrix = Transform2D::identity();
                self.state.line_matrix = Transform2D::identity();
            }
            Op::EndText => (),
            Op::CharSpacing { char_space } => self.state.char_spacing = char_space,
            Op::WordSpacing { word_space } => self.state.word_spacing = word_space,
            Op::TextScaling { horiz_scale } => self.state.horizontal_scale = horiz_scale * 0.01,
            Op::Leading { leading } => self.state.leading = leading,
            Op::TextRise { rise } => self.state.rise = rise,
            Op::MoveTextPosition { translation } => {
                self.state.set_both_matrices(
                    Transform2D::translation(translation.x, translation.y)
                        .then(&self.state.line_matrix),
                );
            }
            Op::SetTextMatrix { matrix } => {
                self.state.set_both_matrices(matrix.into());
            }
            Op::TextNewline => {
                self.state.set_both_matrices(
                    Transform2D::translation(0.0, -self.state.leading)
                        .then(&self.state.line_matrix),
                );
            }

            // --- Actual text ---
            Op::TextDraw { text } => {
                self.render_text(text)?;
            }
            Op::TextDrawAdjusted { array } => {
                for item in array {
                    match item {
                        TextDrawAdjusted::Text(text) => {
                            self.render_text(text)?;
                        }
                        TextDrawAdjusted::Spacing(delta) => {
                            self.state.advance(
                                -delta * self.state.horizontal_scale * self.state.font_size
                                    / 1000.0,
                            );
                        }
                    }
                }
            }

            // --- Xobject ---
            Op::XObject { name } => {
                self.handle_xobject(name)?;
            }

            _ => {}
        };
        Ok(())
    }
}
