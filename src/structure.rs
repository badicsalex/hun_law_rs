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

use std::{collections::BTreeMap, fmt::Debug};

use anyhow::{anyhow, bail, Result};
use chrono::NaiveDate;
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::{
    identifier::{
        ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, IdentifierCommon,
        NumericIdentifier, ParagraphIdentifier, PrefixedAlphabeticIdentifier,
    },
    reference::Reference,
    semantic_info::SemanticInfo,
    util::{debug::DebugContextString, hun_str::FromHungarianString, indentedline::IndentedLine},
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contained_abbreviations: BTreeMap<String, ActIdentifier>,
    pub children: Vec<ActChild>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum ActChild {
    StructuralElement(StructuralElement),
    Subtitle(Subtitle),
    Article(Article),
}

impl Act {
    pub fn articles(&self) -> impl Iterator<Item = &Article> {
        self.children.iter().filter_map(|c| {
            if let ActChild::Article(article) = c {
                Some(article)
            } else {
                None
            }
        })
    }
    pub fn articles_mut(&mut self) -> impl Iterator<Item = &mut Article> {
        self.children.iter_mut().filter_map(|c| {
            if let ActChild::Article(article) = c {
                Some(article)
            } else {
                None
            }
        })
    }
    pub fn article(&self, article_id: ArticleIdentifier) -> Option<&Article> {
        self.articles().find(|a| a.identifier == article_id)
    }
    pub fn article_mut(&mut self, article_id: ArticleIdentifier) -> Option<&mut Article> {
        self.articles_mut().find(|a| a.identifier == article_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralElement {
    pub identifier: NumericIdentifier,
    pub title: String,
    pub element_type: StructuralElementType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_change: Option<LastChange>,
}

impl StructuralElement {
    pub fn header_string(&self) -> Result<String> {
        Ok(match self.element_type {
            StructuralElementType::Book => {
                format!("{} KÖNYV", self.identifier.to_hungarian()?.to_uppercase())
            }
            StructuralElementType::Part { is_special } => {
                if is_special {
                    if self.identifier == 1.into() {
                        "ÁLTALÁNOS RÉSZ"
                    } else if self.identifier == 2.into() {
                        "KÜLÖNÖS RÉSZ"
                    } else if self.identifier == 3.into() {
                        "ZÁRÓ RÉSZ"
                    } else {
                        bail!("Invalid special part")
                    }
                    .to_string()
                } else {
                    format!("{} RÉSZ", self.identifier.to_hungarian()?.to_uppercase())
                }
            }
            StructuralElementType::Title => format!("{}. CÍM", self.identifier.to_roman()?),
            StructuralElementType::Chapter => format!("{}. FEJEZET", self.identifier.to_roman()?),
        })
    }
}

// Separate type from structural elements because of the optional identifier
// and the fact that there are some other special handling around it.

// Guaranteed to start with uppercase
// For older acts, there is no number, only a text.
// Example:
// 17. Az alcím
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subtitle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<NumericIdentifier>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_change: Option<LastChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructuralElementType {
    // Example: NYOLCADIK KÖNYV
    Book,

    // Example: MÁSODIK RÉSZ, KÜLÖNÖS RÉSZ
    Part {
        // Used for the three-part 'ÁLTALÁNOS RÉSZ', 'KÜLÖNÖS RÉSZ', 'ZÁRÓ RÉSZ' version
        // When true, identifier is a number between 1-3, and conversions have to be done on parsing and printing
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
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
            _ => Err(anyhow!("{id} is not a special part")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub identifier: ArticleIdentifier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub children: Vec<Paragraph>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_change: Option<LastChange>,
}

impl Article {
    pub fn header_string(&self) -> String {
        format!("{}. §", self.identifier)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubArticleElement<IdentifierType, ChildrenType>
where
    IdentifierType: IdentifierCommon,
    ChildrenType: ChildrenCommon,
{
    // Note: no serde(default) here, because IdentifierType doesn't usually have a default.
    // Except for paragraphs, which is an Option<NumericIdentifier>.
    // Fortunately (?) serde automatically adds "default" to Option type fields.
    // (See the absolute _madness_ in serde::__private::de::missing_field)
    #[serde(skip_serializing_if = "IdentifierCommon::is_empty")]
    pub identifier: IdentifierType,
    pub body: SAEBody<ChildrenType>,
    #[serde(default, skip_serializing_if = "SemanticInfo::is_empty")]
    pub semantic_info: SemanticInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_change: Option<LastChange>,
}

pub trait ChildrenCommon: Clone + PartialEq + Eq {
    fn is_empty(&self) -> bool;
    fn parent_type_name() -> &'static str;
}

impl<IT: IdentifierCommon, CT: ChildrenCommon> SubArticleElement<IT, CT> {
    /// Returns true if:
    /// - the body is text and it's empty,
    /// - it does not have any children, or
    /// - all its children is empty (recursively)
    pub fn is_empty(&self) -> bool {
        match &self.body {
            SAEBody::Text(t) => t.is_empty(),
            SAEBody::Children { children, .. } => children.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SAEBody<ChildrenType: ChildrenCommon> {
    Text(String),
    Children {
        intro: String,
        children: ChildrenType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        wrap_up: Option<String>,
    },
}

impl<T: ChildrenCommon> From<&str> for SAEBody<T> {
    fn from(s: &str) -> Self {
        SAEBody::Text(s.to_owned())
    }
}

impl<T: ChildrenCommon> From<String> for SAEBody<T> {
    fn from(s: String) -> Self {
        SAEBody::Text(s)
    }
}

pub type Paragraph = SubArticleElement<ParagraphIdentifier, ParagraphChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum ParagraphChildren {
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    QuotedBlock(Vec<QuotedBlock>),
    BlockAmendment(BlockAmendment),
    StructuralBlockAmendment(StructuralBlockAmendment),
}

impl ChildrenCommon for ParagraphChildren {
    fn is_empty(&self) -> bool {
        match self {
            ParagraphChildren::AlphabeticPoint(x) => x.iter().all(|c| c.is_empty()),
            ParagraphChildren::NumericPoint(x) => x.iter().all(|c| c.is_empty()),
            ParagraphChildren::QuotedBlock(x) => x.is_empty(),
            ParagraphChildren::BlockAmendment(x) => x.is_empty(),
            ParagraphChildren::StructuralBlockAmendment(x) => x.is_empty(),
        }
    }

    fn parent_type_name() -> &'static str {
        "Paragraph"
    }
}

pub type AlphabeticPoint = SubArticleElement<AlphabeticIdentifier, AlphabeticPointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum AlphabeticPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
}

impl ChildrenCommon for AlphabeticPointChildren {
    fn is_empty(&self) -> bool {
        match self {
            AlphabeticPointChildren::AlphabeticSubpoint(x) => x.iter().all(|c| c.is_empty()),
            AlphabeticPointChildren::NumericSubpoint(x) => x.iter().all(|c| c.is_empty()),
        }
    }

    fn parent_type_name() -> &'static str {
        "AlphabeticPoint"
    }
}

pub type NumericPoint = SubArticleElement<NumericIdentifier, NumericPointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum NumericPointChildren {
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
}

impl ChildrenCommon for NumericPointChildren {
    fn is_empty(&self) -> bool {
        match self {
            NumericPointChildren::AlphabeticSubpoint(x) => x.iter().all(|c| c.is_empty()),
        }
    }
    fn parent_type_name() -> &'static str {
        "NumericPoint"
    }
}

pub type AlphabeticSubpoint =
    SubArticleElement<PrefixedAlphabeticIdentifier, AlphabeticSubpointChildren>;
// Creating different empty enums is necessary to distinguish between this class and NumericSubpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphabeticSubpointChildren {}

impl ChildrenCommon for AlphabeticSubpointChildren {
    fn is_empty(&self) -> bool {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
    fn parent_type_name() -> &'static str {
        "AlphabeticSubpoint"
    }
}

pub type NumericSubpoint = SubArticleElement<NumericIdentifier, NumericSubpointChildren>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericSubpointChildren {}

impl ChildrenCommon for NumericSubpointChildren {
    fn is_empty(&self) -> bool {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
    fn parent_type_name() -> &'static str {
        "NumericSubpoint"
    }
}

impl<IT, CT> Debug for SubArticleElement<IT, CT>
where
    IT: IdentifierCommon + std::fmt::Debug,
    CT: ChildrenCommon + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct(CT::parent_type_name());
        ds.field("identifier", &self.identifier);
        match &self.body {
            SAEBody::Text(text) => {
                ds.field("body", text);
            }
            SAEBody::Children {
                intro,
                children,
                wrap_up,
            } => {
                ds.field("intro", intro);
                ds.field("children", children);
                if let Some(wrap_up) = wrap_up {
                    ds.field("wrap_up", &wrap_up);
                };
            }
        }
        if !self.semantic_info.is_empty() {
            ds.field("si", &self.semantic_info);
        }
        ds.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotedBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    pub lines: Vec<IndentedLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_up: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockAmendment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    pub children: BlockAmendmentChildren,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_up: Option<String>,
}

impl BlockAmendment {
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromVariants)]
pub enum BlockAmendmentChildren {
    Paragraph(Vec<Paragraph>),
    AlphabeticPoint(Vec<AlphabeticPoint>),
    NumericPoint(Vec<NumericPoint>),
    AlphabeticSubpoint(Vec<AlphabeticSubpoint>),
    NumericSubpoint(Vec<NumericSubpoint>),
}

impl ChildrenCommon for BlockAmendmentChildren {
    fn is_empty(&self) -> bool {
        match self {
            BlockAmendmentChildren::Paragraph(x) => x.iter().all(|c| c.is_empty()),
            BlockAmendmentChildren::AlphabeticPoint(x) => x.iter().all(|c| c.is_empty()),
            BlockAmendmentChildren::NumericPoint(x) => x.iter().all(|c| c.is_empty()),
            BlockAmendmentChildren::AlphabeticSubpoint(x) => x.iter().all(|c| c.is_empty()),
            BlockAmendmentChildren::NumericSubpoint(x) => x.iter().all(|c| c.is_empty()),
        }
    }

    fn parent_type_name() -> &'static str {
        "BlockAmendment"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralBlockAmendment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    pub children: Vec<ActChild>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_up: Option<String>,
}

impl StructuralBlockAmendment {
    pub fn is_empty(&self) -> bool {
        // TODO: This is not actually recursive.
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastChange {
    pub date: NaiveDate,
    /// The act part that casued the change. None means
    /// something automatic (like auto-repealing amendments)
    pub cause: Option<Reference>,
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

impl<IT, CT> DebugContextString for SubArticleElement<IT, CT>
where
    IT: IdentifierCommon + std::fmt::Debug,
    CT: ChildrenCommon,
{
    fn debug_ctx(&self) -> String {
        format!("{} {:?}", CT::parent_type_name(), self.identifier)
    }
}

impl DebugContextString for ActChild {
    fn debug_ctx(&self) -> String {
        match self {
            ActChild::StructuralElement(x) => x.debug_ctx(),
            ActChild::Subtitle(x) => x.debug_ctx(),
            ActChild::Article(x) => x.debug_ctx(),
        }
    }
}

impl DebugContextString for StructuralElement {
    fn debug_ctx(&self) -> String {
        // TODO: Proper pretty format
        format!("{self:?}")
    }
}

impl DebugContextString for Subtitle {
    fn debug_ctx(&self) -> String {
        // TODO: Proper pretty format
        format!("{self:?}")
    }
}

pub trait SAEHeaderString {
    fn header_string(&self) -> String;
}

impl SAEHeaderString for Paragraph {
    fn header_string(&self) -> String {
        if self.identifier.is_empty() {
            String::new()
        } else {
            format!("({})", self.identifier)
        }
    }
}

impl SAEHeaderString for NumericPoint {
    fn header_string(&self) -> String {
        format!("{}.", self.identifier)
    }
}

impl SAEHeaderString for AlphabeticPoint {
    fn header_string(&self) -> String {
        format!("{})", self.identifier)
    }
}

impl SAEHeaderString for NumericSubpoint {
    fn header_string(&self) -> String {
        format!("{}.", self.identifier)
    }
}

impl SAEHeaderString for AlphabeticSubpoint {
    fn header_string(&self) -> String {
        format!("{})", self.identifier)
    }
}
