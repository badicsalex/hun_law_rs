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
use log::debug;
use pdf_extract_fhl as pdf_extract;
use serde::Serialize;

use crate::util::indentedline::{IndentedLine, IndentedLinePart, EMPTY_LINE};

const SAME_LINE_EPSILON: f64 = 0.5;
const ADDITIONAL_EMPTY_LINE_THRESHOLD: f64 = 16.0;
const DEFAULT_WIDTH_OF_SPACE: f64 = 0.25;
const SPACE_DETECTION_THRESHOLD_RATIO: f64 = 0.5;
const JUSTIFIED_DETECTION_THRESHOLD_RATIO: f64 = 0.8;

#[derive(Debug)]
struct PositionedChar {
    x: f64,
    y: f64,
    width: f64,
    width_of_space: f64,
    bold: bool,
    content: char,
}

#[derive(Debug)]
struct PageOfPositionedChars {
    chars: Vec<PositionedChar>,
}

#[derive(Debug, Default)]
struct PdfExtractor {
    pages: Vec<PageOfPositionedChars>,
    width_of_space: f64,
    current_font_is_bold: bool,
    crop: CropBox,
}

#[derive(Debug, Serialize)]
pub struct PageOfLines {
    pub lines: Vec<IndentedLine>,
}

/// Box in PDF coorinates
///
/// Coordinates start from bottom left. Unit is 'point', there are 72 'points' in an inch
/// An A4 page is 595 x 842 points
/// A typical margin is 0.75 inches.
#[derive(Debug, Clone)]
pub struct CropBox {
    pub bottom: f64,
    pub left: f64,
    pub top: f64,
    pub right: f64,
}

impl CropBox {
    fn is_inside(&self, x: f64, y: f64) -> bool {
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

impl PdfExtractor {
    pub fn new(crop: CropBox) -> Self {
        Self {
            crop,
            ..Default::default()
        }
    }

    fn consolidate_line(
        mut chars: Vec<PositionedChar>,
        estimated_right_margin: f64,
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
                        dx: threshold_to_space - prev_x,
                        content: ' ',
                        bold: current_char.bold,
                    });
                    prev_x = threshold_to_space;
                }
            }
            result.push(IndentedLinePart {
                dx: current_char.x - prev_x,
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

    fn consolidate_page(page: PageOfPositionedChars) -> Result<PageOfLines> {
        let estimated_right_margin = page
            .chars
            .iter()
            .fold(0.0_f64, |acc, c| acc.max(c.x + c.width));
        let mut result = Vec::<IndentedLine>::new();
        let mut chars = page.chars;
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
                result.push(Self::consolidate_line(current_line, estimated_right_margin));
                // Add empty line on a "big-enough gap"
                // Should be based on actual font height, but this is
                // good enough for the rest of the parsing steps.
                if y_diff > ADDITIONAL_EMPTY_LINE_THRESHOLD {
                    result.push(EMPTY_LINE);
                }
                current_line = vec![current_char];
            }
        }
        result.push(Self::consolidate_line(current_line, estimated_right_margin));
        Ok(PageOfLines { lines: result })
    }

    pub fn get_parsed_pages(self) -> Result<Vec<PageOfLines>> {
        self.pages.into_iter().map(Self::consolidate_page).collect()
    }
}

impl pdf_extract::OutputDev for PdfExtractor {
    fn begin_page(
        &mut self,
        page_num: u32,
        _media_box: &pdf_extract::MediaBox,
        _art_box: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), pdf_extract::OutputError> {
        self.pages.push(PageOfPositionedChars { chars: Vec::new() });
        assert_eq!(
            (page_num as usize),
            self.pages.len(),
            "Fully linear page numbers during parsing are assumed"
        );
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &pdf_extract::Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        cid: pdf_extract::CharCode,
        char: &str,
    ) -> Result<(), pdf_extract::OutputError> {
        let width = (width * font_size) * trm.m11;
        let width_of_space = self.width_of_space * trm.m11 * font_size;
        let x_start = trm.m31;
        let y = trm.m32;

        if cid < 256 && char.len() == 1 {
            // TODO: horrible workaround for weird InDesign quirk, where in some cases,
            // the casing is wrong for several characters in the ToUnicode map.
            // Since all of these documents (I hope) have ActualText tags, this could be solved
            // by handling the ActualText in the pdf extractor level.
            let cid_char = LATIN2_CHARS[cid as usize];
            let mut cid_str_buf = [0; 4];
            let cid_str = cid_char.encode_utf8(&mut cid_str_buf);
            if cid_char != '�' && cid_str.to_uppercase() == char.to_uppercase() {
                if cid_char != ' ' && self.crop.is_inside(x_start, y) {
                    self.pages.last_mut().unwrap().chars.push(PositionedChar {
                        x: x_start,
                        y,
                        width,
                        width_of_space,
                        content: cid_char,
                        bold: self.current_font_is_bold,
                    });
                }
                return Ok(());
            }
        }

        // Horrible hack to separate ligatures into graphemes.
        // We don't really need to be exact, this 'x' information will probably not be used
        let x_step = width / (char.chars().count() as f64);

        let new_positioned_chars_iter = char
            .chars()
            .enumerate()
            .map(|(i, raw_char)| PositionedChar {
                x: x_start + x_step * (i as f64),
                y,
                width,
                width_of_space,
                content: fix_character_coding_quirks(raw_char),
                bold: self.current_font_is_bold,
            })
            .filter(|p| p.content != ' ' && self.crop.is_inside(p.x, p.y));
        self.pages
            .last_mut()
            .unwrap()
            .chars
            .extend(new_positioned_chars_iter);
        Ok(())
    }

    fn begin_word(
        &mut self,
        font: &dyn pdf_extract::PdfFont,
    ) -> Result<(), pdf_extract::OutputError> {
        self.width_of_space = font.get_width(32).unwrap_or(0.0) / 1000.;
        if self.width_of_space == 0.0 || self.width_of_space == 1.0 {
            debug!("Had to use default space width for font");
            self.width_of_space = DEFAULT_WIDTH_OF_SPACE;
        };
        let base_font = font.get_basefont();
        self.current_font_is_bold = base_font.contains("bold") || base_font.contains("Bold");
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }
}

fn fix_character_coding_quirks(c: char) -> char {
    match c {
        'Õ' => 'Ő', // Note the ~ on top of the first ő
        'õ' => 'ő', // Note the ~ on top of the first ő
        'Û' => 'Ű', // Note the ^ on top of the first ű
        'û' => 'ű', // Note the ^ on top of the first ű
        '\u{a0}' => ' ',
        _ => c,
    }
}

/// Compare floats but only for somewhat correct sorting.
///
/// Does not care about equal values or NaNs
fn compare_float_for_sorting(f1: f64, f2: f64) -> std::cmp::Ordering {
    if f1 < f2 {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}

pub fn parse_pdf(buffer: &[u8], crop: CropBox) -> Result<Vec<PageOfLines>> {
    let document = pdf_extract::Document::load_mem(buffer)?;
    let mut output = PdfExtractor::new(crop);
    pdf_extract::output_doc(&document, &mut output)?;
    output.get_parsed_pages()
}

const LATIN2_CHARS: [char; 256] = [
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', ' ', '!', '\'', '#', '$', '%',
    '&', '\"', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^',
    '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
    'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', 'Ą', '˘', 'Ł', '¤', 'Ľ', 'Ś', '§', '¨', 'Š', 'Ş',
    'Ť', 'Ź', '�', 'Ž', 'Ż', '°', 'ą', '˛', 'ł', '´', 'ľ', 'ś', 'ˇ', '¸', 'š', 'ş', 'ť', 'ź', '˝',
    'ž', 'ż', 'Ŕ', 'Á', 'Â', 'Ă', 'Ä', 'Ĺ', 'Ć', 'Ç', 'Č', 'É', 'Ę', 'Ë', 'Ě', 'Í', 'Î', 'Ď', 'Đ',
    'Ń', 'Ň', 'Ó', 'Ô', 'Ő', 'Ö', '×', 'Ř', 'Ů', 'Ú', 'Ű', 'Ü', 'Ý', 'Ţ', 'ß', 'ŕ', 'á', 'â', 'ă',
    'ä', 'ĺ', 'ć', 'ç', 'č', 'é', 'ę', 'ë', 'ě', 'í', 'î', 'ď', 'đ', 'ń', 'ň', 'ó', 'ô', 'ő', 'ö',
    '÷', 'ř', 'ů', 'ú', 'ű', 'ü', 'ý', 'ţ', '˙',
];
