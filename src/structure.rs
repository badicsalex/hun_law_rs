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

pub enum ActChild {
    StructuralElement(StructuralElement),
    Article(Article)
}

pub struct Act {
    pub identifier: String,
    pub subject: String,
    pub preamble: String,
    pub children: Vec<ActChild>,
}

pub struct StructuralElement {
    pub identifier: String,
    pub title: String,
    pub element_type: StructuralElementType,
}

pub enum StructuralElementType {
    Book,
    Part,
    Title,
    Chapter,
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

pub enum ParagraphChildren {
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    QuotedBlock(Vec<QuotedBlock>),
    BlockAmendment(Vec<BlockAmendment>)
}
pub type Paragraph = SubArticleElement<ParagraphChildren>;

pub enum AlphabeticPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
}
pub type AlphabeticPoint = SubArticleElement<AlphabeticPointChildren>;

pub enum NumericPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
}
pub type NumericPoint = SubArticleElement<NumericPointChildren>;

// Creating different empty enums is necessary to distinguish between this class and NumericSubpoint
pub enum AlphabeticSubpointChildren {}
pub type AlphabeticSubpoint = SubArticleElement<AlphabeticSubpointChildren>;

pub enum NumericSubpointChildren {}
pub type NumericSubpoint = SubArticleElement<NumericSubpointChildren>;

pub struct QuotedBlock {
    pub lines: Vec<IndentedLine>,
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

pub struct BlockAmendment {
    pub children: Vec<BlockAmendmentChild>,
}
