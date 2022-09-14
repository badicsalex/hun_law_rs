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

use std::io::Write;

use anyhow::Result;

use crate::{
    identifier::IdentifierCommon,
    semantic_info::{OutgoingReference, SemanticInfo},
    structure::{
        Act, ActChild, AlphabeticPointChildren, AlphabeticSubpointChildren, Article,
        BlockAmendment, BlockAmendmentChildren, NumericPointChildren, NumericSubpointChildren,
        ParagraphChildren, QuotedBlock, SAEBody, SAEHeaderString, StructuralBlockAmendment,
        StructuralElement, SubArticleElement, Subtitle,
    },
};

#[derive(Debug, Clone)]
pub struct TextOutputParams {
    pub width: usize,
    indentation_level: usize,
    indent_next_line: bool,
    is_colored: bool,
}

pub trait TextOutput {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()>;
}

impl TextOutput for Act {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        // TODO: header
        params.write_header(
            writer,
            &format!(
                "{} {}",
                params.colorize(self.identifier.to_string(), BOLD),
                self.subject
            ),
        )?;
        params.write_newline(writer)?;
        params.write_newline(writer)?;
        params.write_wrapped_line(writer, &self.preamble)?;
        params.write_newline(writer)?;
        let mut last_was_article = true;
        for child in &self.children {
            let this_is_article = matches!(child, ActChild::Article(_));
            if !last_was_article || !this_is_article {
                params.write_newline(writer)?;
            }
            child.write_as_text(writer, params.clone())?;
            last_was_article = this_is_article;
        }
        Ok(())
    }
}

impl TextOutput for ActChild {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()> {
        match self {
            ActChild::StructuralElement(x) => x.write_as_text(writer, params),
            ActChild::Subtitle(x) => x.write_as_text(writer, params),
            ActChild::Article(x) => x.write_as_text(writer, params),
        }
    }
}

impl TextOutput for StructuralElement {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        params.write_wrapped_line(writer, &params.colorize(self.header_string()?, ITALIC))?;
        if !self.title.is_empty() {
            params.write_wrapped_line(writer, &params.colorize(&self.title, ITALIC))?;
        }
        Ok(())
    }
}

impl TextOutput for Subtitle {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        if let Some(identifier) = self.identifier {
            params.write_wrapped_line(
                writer,
                &params.colorize(format!("{}. {}", identifier, self.title), BOLD),
            )?;
        } else {
            params.write_wrapped_line(writer, &params.colorize(&self.title, BOLD))?;
        }
        Ok(())
    }
}

impl TextOutput for Article {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        params.write_header(
            writer,
            &params.colorize(format!("{:<10}", self.header_string()), BOLD),
        )?;
        let mut params = params.indented().indented();
        if let Some(title) = &self.title {
            params.write_wrapped_line(writer, &format!("     [{}]", title))?
        }
        self.children.write_as_text(writer, params.clone())?;
        Ok(())
    }
}

impl<IT, CT> TextOutput for SubArticleElement<IT, CT>
where
    SubArticleElement<IT, CT>: SAEHeaderString,
    IT: IdentifierCommon,
    CT: TextOutput,
{
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        params.write_header(writer, &format!("{:<5}", self.header_string()))?;
        let mut params = params.indented();
        match &self.body {
            SAEBody::Text(text) => {
                params.write_wrapped_line(
                    writer,
                    &colorize_sae_string(text, &params, &self.semantic_info),
                )?;
            }
            SAEBody::Children {
                intro,
                children,
                wrap_up,
            } => {
                params.write_wrapped_line(
                    writer,
                    &colorize_sae_string(intro, &params, &self.semantic_info),
                )?;
                children.write_as_text(writer, params.clone())?;
                if let Some(wrap_up) = wrap_up {
                    params.write_wrapped_line(writer, wrap_up)?;
                }
            }
        }
        Ok(())
    }
}

impl TextOutput for ParagraphChildren {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()> {
        match self {
            ParagraphChildren::AlphabeticPoint(x) => x.write_as_text(writer, params),
            ParagraphChildren::NumericPoint(x) => x.write_as_text(writer, params),
            ParagraphChildren::QuotedBlock(x) => x.write_as_text(writer, params),
            ParagraphChildren::BlockAmendment(x) => x.write_as_text(writer, params),
            ParagraphChildren::StructuralBlockAmendment(x) => x.write_as_text(writer, params),
        }
    }
}

impl TextOutput for AlphabeticPointChildren {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()> {
        match self {
            AlphabeticPointChildren::AlphabeticSubpoint(x) => x.write_as_text(writer, params),
            AlphabeticPointChildren::NumericSubpoint(x) => x.write_as_text(writer, params),
        }
    }
}

impl TextOutput for NumericPointChildren {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()> {
        match self {
            NumericPointChildren::AlphabeticSubpoint(x) => x.write_as_text(writer, params),
        }
    }
}

impl TextOutput for AlphabeticSubpointChildren {
    fn write_as_text(&self, _writer: &mut impl Write, _params: TextOutputParams) -> Result<()> {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
}

impl TextOutput for NumericSubpointChildren {
    fn write_as_text(&self, _writer: &mut impl Write, _params: TextOutputParams) -> Result<()> {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
}

impl TextOutput for QuotedBlock {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        if let Some(intro) = &self.intro {
            params.write_wrapped_line(writer, intro)?;
        };
        params.write_wrapped_line(writer, "„")?;
        let min_indent = self
            .lines
            .iter()
            .map(|l| l.indent())
            .filter(|i| *i > 0.0)
            .fold(1000.0, |a, b| if a > b { b } else { a });
        for line in &self.lines {
            let indent = ((line.indent() - min_indent) * 0.2) as usize + 5;
            params.write_header(writer, &" ".repeat(indent))?;
            params.write_header(writer, line.content())?;
            params.write_newline(writer)?;
        }
        params.write_wrapped_line(writer, "”")?;
        if let Some(wrap_up) = &self.wrap_up {
            params.write_wrapped_line(writer, wrap_up)?;
        };
        Ok(())
    }
}

impl TextOutput for BlockAmendmentChildren {
    fn write_as_text(&self, writer: &mut impl Write, params: TextOutputParams) -> Result<()> {
        match self {
            BlockAmendmentChildren::Paragraph(x) => x.write_as_text(writer, params),
            BlockAmendmentChildren::AlphabeticPoint(x) => x.write_as_text(writer, params),
            BlockAmendmentChildren::NumericPoint(x) => x.write_as_text(writer, params),
            BlockAmendmentChildren::AlphabeticSubpoint(x) => x.write_as_text(writer, params),
            BlockAmendmentChildren::NumericSubpoint(x) => x.write_as_text(writer, params),
        }
    }
}

impl TextOutput for BlockAmendment {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        if let Some(intro) = &self.intro {
            params.write_wrapped_line(writer, &params.colorize(format!("({})", intro), ITALIC))?;
        };
        params.write_wrapped_line(writer, "„")?;
        self.children.write_as_text(writer, params.indented())?;
        params.write_wrapped_line(writer, "”")?;
        if let Some(wrap_up) = &self.wrap_up {
            params
                .write_wrapped_line(writer, &params.colorize(format!("({})", wrap_up), ITALIC))?;
        };
        Ok(())
    }
}

impl TextOutput for StructuralBlockAmendment {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        if let Some(intro) = &self.intro {
            params.write_wrapped_line(writer, &params.colorize(format!("({})", intro), ITALIC))?;
        };
        params.write_wrapped_line(writer, "„")?;
        self.children.write_as_text(writer, params.indented())?;
        params.write_wrapped_line(writer, "”")?;
        if let Some(wrap_up) = &self.wrap_up {
            params
                .write_wrapped_line(writer, &params.colorize(format!("({})", wrap_up), ITALIC))?;
        };
        Ok(())
    }
}

impl<T: TextOutput> TextOutput for Vec<T> {
    fn write_as_text(&self, writer: &mut impl Write, mut params: TextOutputParams) -> Result<()> {
        for element in self {
            element.write_as_text(writer, params.clone())?;
            params.indent_next_line = true;
        }
        Ok(())
    }
}

impl Default for TextOutputParams {
    fn default() -> Self {
        Self {
            width: 105,
            indentation_level: 0,
            indent_next_line: true,
            is_colored: false,
        }
    }
}

impl TextOutputParams {
    fn write_indent_if_needed(&self, writer: &mut impl Write) -> Result<()> {
        if self.indent_next_line {
            for _ in 0..self.indentation_level {
                writer.write_all(b"     ")?
            }
        }
        Ok(())
    }

    fn write_header(&mut self, writer: &mut impl Write, text: &str) -> Result<()> {
        self.write_indent_if_needed(writer)?;
        writer.write_all(text.as_bytes())?;
        self.indent_next_line = false;
        Ok(())
    }

    fn write_wrapped_line(&mut self, writer: &mut impl Write, text: &str) -> Result<()> {
        for line in textwrap::wrap(text, self.width - self.indentation_level * 5) {
            self.write_indent_if_needed(writer)?;
            writer.write_all(line.as_bytes())?;
            self.write_newline(writer)?;
        }
        Ok(())
    }

    fn write_newline(&mut self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(b"\n")?;
        self.indent_next_line = true;
        Ok(())
    }

    fn colorize(&self, text: impl std::fmt::Display, code: &str) -> String {
        if self.is_colored {
            format!("\x1B[{}m{}\x1B[0m", code, text)
        } else {
            text.to_string()
        }
    }

    pub fn indented(&self) -> Self {
        Self {
            indentation_level: self.indentation_level + 1,
            ..self.clone()
        }
    }

    pub fn colored(&self) -> Self {
        Self {
            is_colored: true,
            ..self.clone()
        }
    }
}

fn colorize_sae_string(
    text: &str,
    params: &TextOutputParams,
    semantic_info: &SemanticInfo,
) -> String {
    let default_color = if semantic_info.special_phrase.is_some() {
        "33"
    } else {
        "0"
    };
    split_string_at_references(text, &semantic_info.outgoing_references)
        .into_iter()
        .map(|(t, c)| {
            if c {
                params.colorize(t, "36")
            } else {
                params.colorize(t, default_color)
            }
        })
        .collect()
}

fn split_string_at_references<'a>(
    text: &'a str,
    outgoing_refs: &[OutgoingReference],
) -> Vec<(&'a str, bool)> {
    let mut result = Vec::new();
    let mut prev_end = 0;
    for outgoing_ref in outgoing_refs {
        result.push((&text[prev_end..outgoing_ref.start], false));
        result.push((&text[outgoing_ref.start..outgoing_ref.end], true));
        prev_end = outgoing_ref.end;
    }
    result.push((&text[prev_end..], false));
    result
}

const BOLD: &str = "1";
const ITALIC: &str = "3";
