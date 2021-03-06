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

use crate::structure::ActIdentifier;
use crate::util::date::Date;
use crate::util::indentedline::IndentedLine;
use crate::{parser::pdf::PageOfLines, util::indentedline::EMPTY_LINE};

use anyhow::{bail, Result};
use lazy_regex::regex_captures;

#[derive(Debug, Default)]
pub struct ActRawText {
    pub identifier: ActIdentifier,
    pub subject: String,
    pub publication_date: Date,
    pub body: Vec<IndentedLine>,
}

struct ActExtractor {
    publication_date: Date,
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
        if let Some((_, year_str, num_str)) = regex_captures!(
            "^([12][09][0-9][0-9]). ??vi ([IVXLC]+). t??rv??ny",
            line.content()
        ) {
            if let Ok(year) = year_str.parse::<i16>() {
                if let Some(number) = roman::from(num_str) {
                    self.current_act.identifier = ActIdentifier { year, number };
                    return ParsingActSubject;
                }
            }
        }
        WaitingForHeaderNewline
    }

    fn parse_act_subject(&mut self, line: &IndentedLine) -> ActExtractionState {
        let subject = &mut self.current_act.subject;
        line.append_to(subject);

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
        // "* A t??rv??nyt az Orsz??ggy??l??s a 2010. november 22-i ??l??snapj??n fogadta el."
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
        // Dr. Schmitt P??l s. k.,     Dr. K??v??r L??szl?? s. k.,
        //  k??zt??rsas??gi eln??k        az Orsz??ggy??l??s eln??ke
        if body.len() > 4
            && body[body.len() - 3].is_empty()
            && (body.last().unwrap().content() == "k??zt??rsas??gi eln??k az Orsz??ggy??l??s eln??ke"
                || body.last().unwrap().content() == "k??zt??rsas??gi eln??k az Orsz??ggy??l??s aleln??ke")
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

const ACTS_SECTION_START: &str = "II. T??rv??nyek";

// These are all prefixes, because there are various ways to line break the longer ones
const ACT_SECTION_STOPS: &[&str] = &[
    "III. Korm??nyrendeletek",
    "IV. A Magyar Nemzeti Bank eln??k??nek rendeletei",
    "V. A Korm??ny tagjainak rendeletei",
    "VI. Az Alkotm??nyb??r??s??g hat??rozatai",
    "VII. A K??ria hat??rozatai",
    // TODO: VIII. ????
    "IX. Hat??rozatok T??ra",
];

fn parse_mk_cover_page(page: &PageOfLines) -> Result<Date> {
    // The expected first page is:

    // MAGYAR K??ZL??NY 71 . sz??m
    //
    // A MAGYAR K??ZT??RSAS??G HIVATALOS LAPJA
    // 2011. j??nius 28., kedd

    if page.lines.len() < 4 {
        bail!("First page too short")
    }
    // TODO: Lets hope justified text detector works, and it's not something like
    // "M A G Y A R K O Z L O N Y"
    if !page.lines[0].content().starts_with("MAGYAR K??ZL??NY") {
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
        bail!("Magyar K??zl??ny PDFs should have at least 2 pages")
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
