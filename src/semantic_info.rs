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

use chrono::NaiveDate;
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::identifier::ActIdentifier;
use crate::reference::{Reference, StructuralReference};
use crate::util::{is_default, IsDefault};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticInfo {
    #[serde(default, skip_serializing_if = "is_default")]
    pub outgoing_references: Vec<OutgoingReference>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub new_abbreviations: Vec<ActIdAbbreviation>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub special_phrase: Option<SpecialPhrase>,
}

impl IsDefault for SemanticInfo {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutgoingReference {
    /// Byte index of the first character of the reference string
    pub start: usize,
    /// Byte index after the last character of the reference string
    pub end: usize,
    pub reference: Reference,
}

impl From<OutgoingReference> for Reference {
    fn from(oref: OutgoingReference) -> Self {
        oref.reference
    }
}

impl<'a> From<&'a OutgoingReference> for &'a Reference {
    fn from(oref: &'a OutgoingReference) -> Self {
        &oref.reference
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActIdAbbreviation {
    pub act_id: ActIdentifier,
    pub abbreviation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromVariants)]
pub enum SpecialPhrase {
    ArticleTitleAmendment(ArticleTitleAmendment),
    BlockAmendment(BlockAmendment),
    EnforcementDate(EnforcementDate),
    Repeal(Repeal),
    TextAmendment(TextAmendment),
    StructuralBlockAmendment(StructuralBlockAmendment),
    StructuralRepeal(StructuralRepeal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleTitleAmendment {
    pub position: Reference,
    pub replacement: TextAmendmentReplacement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockAmendment {
    pub position: Reference,
    pub pure_insertion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextAmendment {
    pub positions: Vec<Reference>,
    pub replacements: Vec<TextAmendmentReplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextAmendmentReplacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnforcementDate {
    #[serde(default, skip_serializing_if = "is_default")]
    pub positions: Vec<Reference>,
    pub date: EnforcementDateType,
    #[serde(default, skip_serializing_if = "is_default")]
    pub inline_repeal: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnforcementDateType {
    Date(NaiveDate),
    DaysAfterPublication(u16),
    DayInMonthAfterPublication {
        #[serde(default, skip_serializing_if = "is_default")]
        month: Option<u8>,
        day: u16,
    },
    Special(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repeal {
    pub positions: Vec<Reference>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub texts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralBlockAmendment {
    pub position: StructuralReference,
    pub pure_insertion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralRepeal {
    pub position: StructuralReference,
}
