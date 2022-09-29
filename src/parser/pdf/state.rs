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
pub struct State {
    // Text
    pub text_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    pub line_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    pub char_spacing: f32,
    pub word_spacing: f32,
    pub horizontal_scale: f32,
    pub leading: f32,
    pub font: Option<Rc<FastFont>>,
    pub font_size: f32,
    pub rise: f32,
    // Graphics
    pub graphics_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
}

impl Default for State {
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
            graphics_matrix: Transform2D::identity(),
        }
    }
}

impl State {
    pub fn set_both_matrices(&mut self, m: Transform2D<f32, PdfSpace, PdfSpace>) {
        self.text_matrix = m;
        self.line_matrix = m;
    }

    pub fn advance(&mut self, delta: f32) {
        self.text_matrix = Transform2D::translation(delta, 0.0).then(&self.text_matrix);
    }
}
