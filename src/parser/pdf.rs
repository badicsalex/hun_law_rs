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

use std::{
    collections::{hash_map, HashMap},
    rc::Rc,
};

use anyhow::{anyhow, bail, Result};
use euclid::Transform2D;
use log::debug;
use pdf::{
    content::PdfSpace,
    encoding::Encoding as PdfEncoding,
    font::{Font, ToUnicodeMap, Widths},
    object::{PageRc, Resolve},
    primitive::PdfString,
};
use pdf_encoding::{glyphname_to_unicode, Encoding};
use serde::Serialize;

use crate::util::indentedline::{IndentedLine, IndentedLinePart, EMPTY_LINE};

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
struct CharCollector {
    chars: Vec<PositionedChar>,
    width_of_space: f32,
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
        if !content.is_ascii_whitespace() && self.crop.is_inside(x, y) {
            println!(
                "Char: {} x: {} y: {} w: {} ws: {}",
                content, x, y, width, width_of_space
            );
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

    fn render_multiple_characters(
        &mut self,
        x: f32,
        y: f32,
        w0: f32,
        horizontal_scale: f32,
        chars: &str,
    ) -> Result<()> {
        // Horrible hack to separate ligatures into graphemes.
        // We don't really need to be exact, this 'x' information will probably not be used
        let x_step = w0 / (chars.chars().count() as f32);
        for (i, c) in chars.chars().enumerate() {
            self.render_character(x + i as f32 * x_step, y, x_step, horizontal_scale, c)?
        }
        Ok(())
    }

    fn render_cid(&mut self, state: &mut State, font: &FastFont, cid: u32) -> Result<()> {
        let rendering_pre_matrix = Transform2D::<f32, PdfSpace, PdfSpace>::new(
            state.horizontal_scale * state.font_size,
            0.0,
            0.0,
            state.font_size,
            0.0,
            state.rise,
        );
        let rendering_matrix = rendering_pre_matrix.then(&state.text_matrix); /* TODO: .then(CTM) */
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

    fn render_text(&mut self, state: &mut State, text: PdfString) -> Result<()> {
        let data = text.as_bytes();
        let font = state
            .font
            .as_ref()
            .ok_or_else(|| anyhow!("Trying to draw text without a font"))?
            .clone();
        if font.is_cid {
            data.chunks_exact(2).try_for_each(|s| -> Result<()> {
                let cid = u16::from_be_bytes(s.try_into()?);
                self.render_cid(state, &font, cid as u32)?;
                Ok(())
            })?;
        } else {
            data.iter()
                .try_for_each(|cid| self.render_cid(state, &font, *cid as u32))?;
        }

        Ok(())
    }

    fn font_changed(&mut self, font: &FastFont) -> Result<()> {
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
fn compare_float_for_sorting(f1: f32, f2: f32) -> std::cmp::Ordering {
    if f1 < f2 {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}

#[derive(Debug)]
struct FastFont {
    cmap: Vec<char>,
    smap: HashMap<u32, String>,
    is_cid: bool,
    is_identity: bool,
    name: Option<String>,
    widths: Widths,
}

#[derive(Debug)]
enum ToUnicodeResult<'a> {
    Unknown,
    Char(char),
    String(&'a str),
}

impl FastFont {
    fn convert(font: &Font, resolve: &impl Resolve) -> Result<Self> {
        let widths = font
            .widths(resolve)
            .map_err(|e| anyhow!("{}", e))?
            .ok_or_else(|| anyhow!("No widths in font {:?}", font))?;
        let mut result = Self {
            cmap: Default::default(),
            smap: Default::default(),
            is_cid: font.is_cid(),
            is_identity: false,
            name: font.name.as_ref().map(|n| n.to_string()),
            widths,
        };
        let encoding = font
            .encoding()
            .ok_or_else(|| anyhow!("No encoding in font {:?}", font))?
            .clone();
        result.process_encoding(&encoding)?;
        if let Some(to_unicode) = font.to_unicode(resolve).transpose()? {
            result.process_unicode_map(&to_unicode);
        }
        Ok(result)
    }

    fn process_encoding(&mut self, encoding: &PdfEncoding) -> Result<()> {
        match encoding.base {
            pdf::encoding::BaseEncoding::StandardEncoding => {
                self.process_base_encoding(&pdf_encoding::STANDARD)
            }
            pdf::encoding::BaseEncoding::SymbolEncoding => {
                self.process_base_encoding(&pdf_encoding::SYMBOL)
            }
            pdf::encoding::BaseEncoding::MacRomanEncoding => {
                self.process_base_encoding(&pdf_encoding::MACROMAN)
            }
            pdf::encoding::BaseEncoding::WinAnsiEncoding => {
                self.process_base_encoding(&pdf_encoding::WINANSI)
            }
            pdf::encoding::BaseEncoding::MacExpertEncoding => {
                self.process_base_encoding(&pdf_encoding::MACEXPERT)
            }
            pdf::encoding::BaseEncoding::IdentityH => {
                self.is_identity = true;
            }
            _ => bail!("Unsupported encoding: {:?}", encoding),
        };
        for (c, s) in encoding.differences.iter() {
            if let Some(uc) = glyphname_to_unicode(s) {
                self.add_mapping(*c, uc);
            }
        }
        Ok(())
    }

    fn process_base_encoding(&mut self, map: &pdf_encoding::ForwardMap) {
        let mut tmp = [0u8; 4];
        for cid in 0..255 {
            if let Some(c) = map.get(cid) {
                self.add_mapping(cid as u32, c.encode_utf8(&mut tmp));
            }
        }
    }

    fn process_unicode_map(&mut self, to_unicode: &ToUnicodeMap) {
        for (cid, s) in to_unicode.iter() {
            self.add_mapping(cid as u32, s)
        }
    }

    fn add_mapping(&mut self, cid: u32, s: &str) {
        if cid > 1000 || s.chars().count() > 1 {
            self.smap.insert(cid, s.to_string());
            return;
        }
        if cid as usize >= self.cmap.len() {
            self.cmap.resize(cid as usize + 1, '\u{0}');
        }
        self.cmap[cid as usize] = s.chars().next().unwrap();
    }

    pub fn to_unicode(&self, cid: u32) -> ToUnicodeResult {
        let mut result = ToUnicodeResult::Unknown;
        if self.is_identity {
            if let Some(c) = char::from_u32(cid) {
                result = ToUnicodeResult::Char(c);
            }
        }
        if let Some(c) = self.cmap.get(cid as usize) {
            if *c != '\u{0}' {
                result = ToUnicodeResult::Char(*c);
            }
        }
        if let Some(s) = self.smap.get(&cid) {
            result = ToUnicodeResult::String(s);
        }
        result
    }
}

#[derive(Debug, Default)]
struct FontCache {
    cache: HashMap<usize, Rc<FastFont>>,
}

impl FontCache {
    fn get(&mut self, font: &Font, resolve: &impl Resolve) -> Result<Rc<FastFont>> {
        let entry = self.cache.entry(font as *const Font as usize);
        Ok(match entry {
            hash_map::Entry::Occupied(e) => e.get().clone(),
            hash_map::Entry::Vacant(e) => {
                let fast_font = Rc::new(FastFont::convert(font, resolve)?);
                e.insert(fast_font).clone()
            }
        })
    }
}

#[derive(Clone, Debug)]
struct State {
    // Text
    text_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    line_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
    char_spacing: f32,
    word_spacing: f32,
    horizontal_scale: f32,
    leading: f32,
    font: Option<Rc<FastFont>>,
    font_size: f32,
    rise: f32,
    // Graphics
    graphics_matrix: Transform2D<f32, PdfSpace, PdfSpace>,
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
        render_page(page, &pdf_file, &mut font_cache, &mut collector);
        result.push(collector.try_into()?);
    }
    Ok(result)
}
