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

use euclid::Transform2D;

use pdf::content::PdfSpace;

use super::font::FastFont;

#[derive(Clone, Debug)]
pub struct TextState {
    pub text_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    pub line_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    pub char_spacing: f32,
    pub word_spacing: f32,
    pub horizontal_scale: f32,
    pub leading: f32,
    pub font: Option<Rc<FastFont>>,
    pub font_size: f32,
    pub rise: f32,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            text_matrix: Transform2D::identity(),
            line_matrix: Transform2D::identity(),
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scale: 1.0,
            leading: 0.0,
            font: None,
            font_size: 0.0,
            rise: 0.0,
        }
    }
}

impl TextState {
    pub fn set_both_matrices(&mut self, m: Transform2D<f32, PdfSpace, PdfSpace>) {
        self.text_matrix = m;
        self.line_matrix = m;
    }

    pub fn advance(&mut self, delta: f32) {
        self.text_matrix = Transform2D::translation(delta, 0.0).then(&self.text_matrix);
    }

    pub fn advance_by_char(&mut self, cid: u32) {
        let w0 = self
            .font
            .as_ref()
            .map_or(0.0, |font| font.widths.get(cid as usize) / 1000.0);
        let spacing = self.char_spacing + if cid == 32 { self.word_spacing } else { 0.0 };
        let tx = (w0 * self.font_size + spacing) * self.horizontal_scale;
        self.advance(tx);
    }

    pub fn rendering_matrix(&self) -> Transform2D<f32, PdfSpace, PdfSpace> {
        let rendering_pre_matrix = Transform2D::<f32, PdfSpace, PdfSpace>::new(
            self.horizontal_scale * self.font_size,
            0.0,
            0.0,
            self.font_size,
            0.0,
            self.rise,
        );
        /* TODO: .then(CTM) */
        rendering_pre_matrix.then(&self.text_matrix)
    }
}
