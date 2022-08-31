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

use std::fmt::Debug;

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::{
    identifier::{
        ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, IsNextFrom, NumericIdentifier,
        PrefixedAlphabeticIdentifier,
    },
    semantic_info::SemanticInfo,
    util::{
        debug::DebugContextString, hun_str::FromHungarianString, indentedline::IndentedLine,
        is_default, IsDefault,
    },
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Act {
    pub identifier: ActIdentifier,
    pub subject: String,
    pub preamble: String,
    pub publication_date: NaiveDate,
    pub children: Vec<ActChild>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum ActChild {
    StructuralElement(StructuralElement),
    Subtitle(Subtitle),
    Article(Article),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralElement {
    pub identifier: NumericIdentifier,
    pub title: String,
    pub element_type: StructuralElementType,
}

// Separate type from structural elements because of the optional identifier
// and the fact that there are some other special handling around it.

// Guaranteed to start with uppercase
// For older acts, there is no number, only a text.
// Example:
// 17. Az alcím
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subtitle {
    #[serde(default, skip_serializing_if = "is_default")]
    pub identifier: Option<NumericIdentifier>,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructuralElementType {
    // Example: NYOLCADIK KÖNYV
    Book,

    // Example: MÁSODIK RÉSZ, KÜLÖNÖS RÉSZ
    Part {
        // Used for the three-part 'ÁLTALÁNOS RÉSZ', 'KÜLÖNÖS RÉSZ', 'ZÁRÓ RÉSZ' version
        // When true, identifier is a number between 1-3, and conversions have to be done on parsing and printing
        #[serde(default, skip_serializing_if = "is_default")]
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
}

impl StructuralElementType {
    pub fn parse_identifier(&self, id: &str) -> Result<NumericIdentifier> {
        match self {
            StructuralElementType::Part { is_special: true } => {
                Self::parse_special_part_identifier(id)
            }
            StructuralElementType::Book | StructuralElementType::Part { is_special: false } => {
                u16::from_hungarian(id).map(NumericIdentifier::from)
            }
            StructuralElementType::Title | StructuralElementType::Chapter => {
                NumericIdentifier::from_roman(id)
            }
        }
    }

    fn parse_special_part_identifier(id: &str) -> Result<NumericIdentifier> {
        match id {
            "ÁLTALÁNOS" => Ok(1.into()),
            "KÜLÖNÖS" => Ok(2.into()),
            "ZÁRÓ" => Ok(3.into()),
            _ => Err(anyhow!("{} is not a special part", id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub identifier: ArticleIdentifier,
    #[serde(default, skip_serializing_if = "is_default")]
    pub title: Option<String>,
    pub children: Vec<Paragraph>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubArticleElement<IdentifierType, ChildrenType>
where
    IdentifierType: IsNextFrom + IsDefault + Sized,
{
    // Note: no serde(default) here, because IdentifierType doesn't usually have a default.
    // Except for paragraphs, which is an Option<NumericIdentifier>.
    // Fortunately (?) serde automatically adds "default" to Option type fields.
    #[serde(skip_serializing_if = "is_default")]
    pub identifier: IdentifierType,
    pub body: SAEBody<ChildrenType>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub semantic_info: SemanticInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SAEBody<ChildrenType> {
    Text(String),
    Children {
        intro: String,
        children: ChildrenType,
        #[serde(default, skip_serializing_if = "is_default")]
        wrap_up: Option<String>,
    },
}

impl<T> From<&str> for SAEBody<T> {
    fn from(s: &str) -> Self {
        SAEBody::Text(s.to_owned())
    }
}

impl<T> From<String> for SAEBody<T> {
    fn from(s: String) -> Self {
        SAEBody::Text(s)
    }
}

pub type Paragraph = SubArticleElement<Option<NumericIdentifier>, ParagraphChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum ParagraphChildren {
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    QuotedBlock(Vec<QuotedBlock>),
    BlockAmendment(BlockAmendment),
}

pub type AlphabeticPoint = SubArticleElement<AlphabeticIdentifier, AlphabeticPointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum AlphabeticPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
}

pub type NumericPoint = SubArticleElement<NumericIdentifier, NumericPointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum NumericPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
}

pub type AlphabeticSubpoint =
    SubArticleElement<PrefixedAlphabeticIdentifier, AlphabeticSubpointChildren>;
// Creating different empty enums is necessary to distinguish between this class and NumericSubpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphabeticSubpointChildren {}

pub type NumericSubpoint = SubArticleElement<NumericIdentifier, NumericSubpointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericSubpointChildren {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotedBlock {
    #[serde(default, skip_serializing_if = "is_default")]
    pub intro: Option<String>,
    pub lines: Vec<IndentedLine>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub wrap_up: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockAmendment {
    #[serde(default, skip_serializing_if = "is_default")]
    pub intro: Option<String>,
    pub children: BlockAmendmentChildren,
    #[serde(default, skip_serializing_if = "is_default")]
    pub wrap_up: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum BlockAmendmentChildren {
    Article(Vec<Article>),
    Paragraph(Vec<Paragraph>),
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
    StructuralElement(Vec<ActChild>),
}

// This trait is a workaround for the following limitations:
// - No inherent associated types
// - Generic type cannot be used as a trait bound
pub trait SAECommon: Sized {
    type IdentifierType: IsNextFrom + Clone + Debug + Eq;
    type ChildrenType;
    fn new(identifier: Self::IdentifierType, body: SAEBody<Self::ChildrenType>) -> Self;
}

impl<IdentifierType, ChildrenType> SAECommon for SubArticleElement<IdentifierType, ChildrenType>
where
    IdentifierType: IsDefault + IsNextFrom + Clone + Debug + Eq,
{
    type IdentifierType = IdentifierType;
    type ChildrenType = ChildrenType;

    fn new(identifier: Self::IdentifierType, body: SAEBody<Self::ChildrenType>) -> Self {
        Self {
            identifier,
            body,
            semantic_info: SemanticInfo::default(),
        }
    }
}

macro_rules! simple_dbg_ctx {
    ($t:ident) => {
        impl DebugContextString for $t {
            fn debug_ctx(&self) -> String {
                format!(concat!(stringify!($t), " {}"), self.identifier)
            }
        }
    };
}

simple_dbg_ctx!(Act);
simple_dbg_ctx!(Article);
simple_dbg_ctx!(AlphabeticPoint);
simple_dbg_ctx!(NumericPoint);
simple_dbg_ctx!(AlphabeticSubpoint);
simple_dbg_ctx!(NumericSubpoint);
impl DebugContextString for Paragraph {
    fn debug_ctx(&self) -> String {
        format!(
            "Paragraph {}",
            self.identifier.map_or("None".to_owned(), |i| i.to_string())
        )
    }
}
