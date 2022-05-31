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

use crate::util::indentedline::IndentedLine;

//  Main act on which all the code was based:
//  61/2009. (XII. 14.) IRM rendelet a jogszabályszerkesztésről
//
//  Structuring levels (36. § (2)), and their Akoma Ntoso counterpart (at least IMO):
//  a) az alpont,                         | subpoint
//  b) a pont,                            | point
//  c) a bekezdés,                        | paragraph
//  d) a szakasz, [a.ka. paragrafus]      | article *
//  e) az alcím,                          | subtitle
//  f) a fejezet,                         | chapter
//  g) a rész és                          | part
//  h) a könyv.                           | book
//
//  Additional levels for non-conformant laws, such as 2013. V (PTK):
//     cím                                | title
//
//  * even though we call this level "sections" in hungarian (was "paragrafus")
//  similar levels are called "section" in UK and US, but "Article" in EU Acts.
//
//  Numbering is non-intuitive:
//  Book 1
//    Part 1
//      Title 1
//        Article 1
//          Paragraph 1
//          Paragraph 2
//      Title 2
//        Article 2
//          Paragraph 1
//            Point a)
//            Point b)
//        Article 3
//          Point a)
//    Part 2
//      Title 3
//        Article 4
//        Article 5
//      Title 4
//        Article 6
//  Book 2
//    Part 1
//      Title 1
//        Article 1
//  ....
//
//  Sometimes numbering are different, especially for older Acts.
//  Also, sometimes a Part has Articles outside Titles (at the beginning)
//  See 2013. V, 3:159. §
//
//  For this reason, (and because they are so useless) we only handle structure levels,
//  as mere bookmarks, and don't use them as a tree or similar.

pub struct Act {
    pub identifier: String,
    pub subject: String,
    pub preamble: String,
    pub children: Vec<ActChild>,
}

pub enum ActChild {
    StructuralElement(StructuralElement),
    Article(Article),
}

pub struct StructuralElement {
    pub identifier: String,
    pub title: String,
    pub element_type: StructuralElementType,
}

pub enum StructuralElementType {
    // Example: NYOLCADIK KÖNYV
    Book,

    // Example: MÁSODIK RÉSZ, KÜLÖNÖS RÉSZ
    Part {
        // Used for the three-part 'ÁLTALÁNOS RÉSZ', 'KÜLÖNÖS RÉSZ', 'ZÁRÓ RÉSZ' version
        // When true, identifier is a number between 1-3, and conversions have to be done on parsing and printing
        is_special: bool,
    },

    // Nonconformant structural type, present only in PTK
    // Example:
    // XXI. CÍM
    Title,

    // Example:
    // II. FEJEZET
    // IV. Fejezet
    // XXIII. fejezet  <=  not conformant, but present in e.g. PTK
    Chapter,

    // Guaranteed to be uppercase
    // For older acts, there is no number, only a text.
    // Example:
    // 17. Az alcím
    Subtitle,
}

pub struct Article {
    pub identifier: String,
    pub title: String,
    pub children: Vec<Paragraph>,
}

pub struct SubArticleElement<ChildrenType> {
    pub identifier: String,
    pub body: SAEBody<ChildrenType>,
}

pub enum SAEBody<ChildrenType> {
    Text(String),
    Children {
        intro: String,
        children: ChildrenType,
        wrap_up: String,
    },
}

pub type Paragraph = SubArticleElement<ParagraphChildren>;
pub enum ParagraphChildren {
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    QuotedBlock(Vec<QuotedBlock>),
    BlockAmendment(Vec<BlockAmendment>),
}

pub type AlphabeticPoint = SubArticleElement<AlphabeticPointChildren>;
pub enum AlphabeticPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
}

pub type NumericPoint = SubArticleElement<NumericPointChildren>;
pub enum NumericPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
}

pub type AlphabeticSubpoint = SubArticleElement<AlphabeticSubpointChildren>;
// Creating different empty enums is necessary to distinguish between this class and NumericSubpoint
pub enum AlphabeticSubpointChildren {}

pub type NumericSubpoint = SubArticleElement<NumericSubpointChildren>;
pub enum NumericSubpointChildren {}

pub struct QuotedBlock {
    pub lines: Vec<IndentedLine>,
}

pub struct BlockAmendment {
    pub children: Vec<BlockAmendmentChild>,
}

pub enum BlockAmendmentChild {
    Article(Article),
    Paragraph(Paragraph),
    AlphabeticPoint(AlphabeticPoint),
    NumericPoint(NumericPoint),
    AlphabeticSubpoint(AlphabeticSubpoint),
    NumericSubpoint(NumericSubpoint),
    StructuralElement(StructuralElement),
}
