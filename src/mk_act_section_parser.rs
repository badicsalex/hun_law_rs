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

use crate::util::date::Date;
use crate::util::indentedline::IndentedLine;
use crate::{pdf_parser::PageOfLines, util::indentedline::EMPTY_LINE};

use anyhow::{bail, Result};
use regex::Regex;

#[derive(Debug, Default)]
pub struct ActIdentifier {
    pub year: i16,
    pub number: i32,
}

impl ToString for ActIdentifier {
    fn to_string(&self) -> String {
        format!(
            "{:?}. évi {}. törvény",
            self.year,
            roman::to(self.number).unwrap()
        )
    }
}

#[derive(Debug, Default)]
pub struct ActRawText {
    pub identifier: ActIdentifier,
    pub subject: String,
    pub publication_date: Date,
    pub body: Vec<IndentedLine>,
}

struct ActExtractor {
    publication_date: Date,
    act_header_regex: Regex,
    current_act: ActRawText,
    result: Vec<ActRawText>,
    state: ActExtractionState,
}

#[derive(Debug)]
enum ActExtractionState {
    WaitingForHeaderNewline,
    WaitingForHeader,
    ParsingActSubject,
    BodyBeforeAsteriskFooter,
    BodyAfterAsteriskFooter,
}
use ActExtractionState::*;

impl ActExtractor {
    fn new(publication_date: Date) -> Self {
        Self {
            current_act: Default::default(),
            state: WaitingForHeaderNewline,
            act_header_regex: Regex::new("^([12][09][0-9][0-9]). évi ([IVXLC]+). törvény").unwrap(),
            publication_date,
            result: Vec::new(),
        }
    }

    fn feed_line(&mut self, line: &IndentedLine) {
        self.state = match self.state {
            WaitingForHeaderNewline => self.wait_for_header_newline(line),
            WaitingForHeader => self.wait_for_header(line),
            ParsingActSubject => self.parse_act_subject(line),
            BodyBeforeAsteriskFooter => self.parse_body_before_footer(line),
            BodyAfterAsteriskFooter => self.parse_body_after_footer(line),
        }
    }

    fn wait_for_header_newline(&self, line: &IndentedLine) -> ActExtractionState {
        if line.is_empty() {
            return WaitingForHeader;
        }

        WaitingForHeaderNewline
    }

    fn wait_for_header(&mut self, line: &IndentedLine) -> ActExtractionState {
        if let Some(captures) = self.act_header_regex.captures(line.content()) {
            if let Ok(year) = captures[1].parse::<i16>() {
                if let Some(number) = roman::from(&captures[2]) {
                    self.current_act.identifier = ActIdentifier { year, number };
                    return ParsingActSubject;
                }
            }
        }
        WaitingForHeaderNewline
    }

    fn parse_act_subject(&mut self, line: &IndentedLine) -> ActExtractionState {
        let subject = &mut self.current_act.subject;
        if !subject.is_empty() {
            subject.push(' ')
        }
        subject.push_str(line.content());

        // TODO: this is a huge hack, because we depend on there always being a footer about
        // when the law or amendment was enacted and by whom.
        // Also let's hope there are no two small laws on a single page
        if subject.ends_with('*') {
            subject.pop();
            return BodyBeforeAsteriskFooter;
        }

        ParsingActSubject
    }

    fn parse_body_before_footer(&mut self, line: &IndentedLine) -> ActExtractionState {
        // State to swallow the following footer:
        // "* A törvényt az Országgyűlés a 2010. november 22-i ülésnapján fogadta el."
        let body = &mut self.current_act.body;
        if line.is_empty()
            && body.len() > 2
            && body[body.len() - 2].is_empty()
            && body[body.len() - 1].content().starts_with('*')
        {
            body.pop(); // Pop the asterisk footer, leave the empty line
            return BodyAfterAsteriskFooter;
        }

        // There might not be an asterisk footer at all before the end of the act,
        // so check for that too in this state.
        match self.parse_body_after_footer(line) {
            // Stay in this state if parse_body_after_footer() didn't do anything funky.
            BodyAfterAsteriskFooter => BodyBeforeAsteriskFooter,

            // Hopefully this other state can deal with the asterisk footer.
            other => other,
        }
    }

    fn parse_body_after_footer(&mut self, line: &IndentedLine) -> ActExtractionState {
        let body = &mut self.current_act.body;
        body.push(line.clone());
        // Example for the actual format of the act footer:

        // [EMPTY]
        // Dr. Schmitt Pál s. k.,     Dr. Kövér László s. k.,
        //  köztársasági elnök        az Országgyűlés elnöke
        if body.len() > 4
            && body[body.len() - 3].is_empty()
            && (body.last().unwrap().content() == "köztársasági elnök az Országgyűlés elnöke"
                || body.last().unwrap().content() == "köztársasági elnök az Országgyűlés alelnöke")
        {
            body.truncate(body.len() - 3);
            self.current_act.publication_date = self.publication_date.clone();
            // take() fills self.current_act with defaults, which is exactly what we want.
            self.result.push(std::mem::take(&mut self.current_act));
            return WaitingForHeaderNewline;
        }

        BodyAfterAsteriskFooter
    }
}

const ACTS_SECTION_START: &str = "II. Törvények";

// These are all prefixes, because there are various ways to line break the longer ones
const ACT_SECTION_STOPS: &[&str] = &[
    "III. Kormányrendeletek",
    "IV. A Magyar Nemzeti Bank elnökének rendeletei",
    "V. A Kormány tagjainak rendeletei",
    "VI. Az Alkotmánybíróság határozatai",
    "VII. A Kúria határozatai",
    // TODO: VIII. ????
    "IX. Határozatok Tára",
];

fn parse_mk_cover_page(page: &PageOfLines) -> Result<Date> {
    // The expected first page is:

    // MAGYAR KÖZLÖNY 71 . szám
    //
    // A MAGYAR KÖZTÁRSASÁG HIVATALOS LAPJA
    // 2011. június 28., kedd

    if page.lines.len() < 4 {
        bail!("First page too short")
    }
    // TODO: Lets hope justified text detector works, and it's not something like
    // "M A G Y A R K O Z L O N Y"
    if !page.lines[0].content().starts_with("MAGYAR KÖZLÖNY") {
        bail!("Wrong header on PDF: {}", page.lines[0].content())
    }
    Date::from_hungarian_string(page.lines[3].content())
}

fn line_is_act_section_start(line: &IndentedLine) -> bool {
    line.is_bold() && line.content() == ACTS_SECTION_START
}

fn line_is_act_section_end(line: &IndentedLine) -> bool {
    line.is_bold()
        && ACT_SECTION_STOPS
            .iter()
            .any(|pat| line.content().starts_with(pat))
}

pub fn parse_mk_pages_into_acts(pages: &[PageOfLines]) -> Result<Vec<ActRawText>> {
    if pages.len() < 2 {
        bail!("Magyar Közlöny PDFs should have at least 2 pages")
    }
    let publication_date = parse_mk_cover_page(&pages[0])?;

    let mut extractor = ActExtractor::new(publication_date);
    let mut extracting = false;
    for page in pages {
        for line in &page.lines {
            if line_is_act_section_start(line) {
                extracting = true;
            } else if line_is_act_section_end(line) {
                extracting = false;
            } else if extracting {
                extractor.feed_line(line);
            }
        }
        // This is where we do away with the "Page" abstraction, and further processing
        // can only use EMPTY_LINE to have some separation info.
        if extracting {
            extractor.feed_line(&EMPTY_LINE);
        }
    }

    Ok(extractor.result)
}
