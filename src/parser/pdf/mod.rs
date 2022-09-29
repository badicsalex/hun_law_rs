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

mod actual_text;
mod collector;
mod font;
mod page;
mod page_of_lines;
mod textstate;
mod util;

use anyhow::Result;

use self::{collector::CharCollector, font::FontCache, page::PageRenderer};

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

pub use page_of_lines::PageOfLines;

pub fn parse_pdf(buffer: &[u8], crop: CropBox) -> Result<Vec<PageOfLines>> {
    let pdf_file = pdf::file::File::from_data(buffer)?;
    let mut font_cache = FontCache::default();
    let mut result = Vec::new();
    for page in pdf_file.pages() {
        let page = page?;
        let mut collector = CharCollector::new(crop.clone());
        PageRenderer::new(page, &pdf_file, &mut font_cache, &mut collector)?.render()?;
        result.push(collector.try_into()?);
    }
    Ok(result)
}
