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

use anyhow::Result;

use super::{collector::CharCollector, font::FastFont, state::State};

#[derive(Debug)]
pub struct ActualTextCollector {
    actual_text: String,
    params: Option<RenderCidParams>,
}

#[derive(Debug)]
pub struct RenderCidParams {
    x: f32,
    y: f32,
    width: f32,
    scaling: f32,
}

impl ActualTextCollector {
    pub fn new(actual_text: String) -> Self {
        Self {
            actual_text,
            params: None,
        }
    }

    pub fn render_cid(&mut self, state: &mut State, font: &FastFont, cid: u32) -> Result<()> {
        let rendering_matrix = state.rendering_matrix();
        let w0 = font.widths.get(cid as usize) / 1000.0;

        if self.params.is_none() {
            self.params = Some(RenderCidParams {
                x: rendering_matrix.m31,
                y: rendering_matrix.m32,
                width: 0.0,
                scaling: rendering_matrix.m11,
            });
        }
        if let Some(params) = &mut self.params {
            params.width = (rendering_matrix.m31 - params.x) + w0 * rendering_matrix.m11
        };

        let spacing = state.char_spacing + if cid == 32 { state.word_spacing } else { 0.0 };
        let tx = (w0 * state.font_size + spacing) * state.horizontal_scale;
        state.advance(tx);
        Ok(())
    }

    pub fn finish(self, collector: &mut CharCollector) -> Result<()> {
        println!("Actual text'd: {}", self.actual_text);
        if let Some(params) = self.params {
            collector.render_multiple_characters(
                params.x,
                params.y,
                params.width,
                params.scaling,
                &self.actual_text,
            )
        } else {
            // XXX: Or not OK?
            Ok(())
        }
    }
}
