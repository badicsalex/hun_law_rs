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
use pdf::{
    encoding::{BaseEncoding, Encoding as PdfEncoding},
    font::{Font, ToUnicodeMap, Widths},
    object::Resolve,
};
use pdf_encoding::glyphname_to_unicode;

#[derive(Debug)]
pub struct FastFont {
    cmap: Vec<char>,
    smap: HashMap<u32, String>,
    is_identity: bool,
    pub is_cid: bool,
    pub name: Option<String>,
    pub widths: Widths,
}

#[derive(Debug)]
pub enum ToUnicodeResult<'a> {
    Unknown,
    Char(char),
    String(&'a str),
}

impl FastFont {
    pub fn convert(font: &Font, resolve: &impl Resolve) -> Result<Self> {
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
            BaseEncoding::StandardEncoding => self.process_base_encoding(&pdf_encoding::STANDARD),
            BaseEncoding::SymbolEncoding => self.process_base_encoding(&pdf_encoding::SYMBOL),
            BaseEncoding::MacRomanEncoding => self.process_base_encoding(&pdf_encoding::MACROMAN),
            BaseEncoding::WinAnsiEncoding => {
                let mut tmp = [0u8; 4];
                for cid in 0..255 {
                    if let Some(c) = pdf_encoding::WINANSI.get(cid) {
                        self.add_mapping(cid as u32, c.encode_utf8(&mut tmp));
                    } else if cid > 30 {
                        // This is a quirk in WinAnsi encoding: some of these
                        // characters are not explicitly defined, but they
                        // are displayed as bullets with the acutal WinAPI
                        // Unfortunately this character is present in some
                        // documents.
                        self.add_mapping(cid as u32, "\u{2022}")
                    }
                }
            }
            BaseEncoding::MacExpertEncoding => self.process_base_encoding(&pdf_encoding::MACEXPERT),
            BaseEncoding::IdentityH => {
                self.is_identity = true;
            }
            BaseEncoding::None => {}
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
pub struct FontCache {
    cache: HashMap<usize, Rc<FastFont>>,
}

impl FontCache {
    pub fn get(&mut self, font: &Font, resolve: &impl Resolve) -> Result<Rc<FastFont>> {
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
