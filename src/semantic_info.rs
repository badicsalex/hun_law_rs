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

use std::collections::BTreeMap;

use chrono::NaiveDate;
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::identifier::ActIdentifier;
use crate::reference::{structural::StructuralReference, Reference};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticInfo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outgoing_references: Vec<OutgoingReference>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub new_abbreviations: BTreeMap<String, ActIdentifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_phrase: Option<SpecialPhrase>,
}

impl SemanticInfo {
    pub fn is_empty(&self) -> bool {
        self.outgoing_references.is_empty()
            && self.new_abbreviations.is_empty()
            && self.special_phrase.is_none()
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromVariants)]
pub enum SpecialPhrase {
    BlockAmendment(BlockAmendment),
    EnforcementDate(EnforcementDate),
    Repeal(Repeal),
    TextAmendment(Vec<TextAmendment>),
    StructuralBlockAmendment(StructuralBlockAmendment),
    StructuralRepeal(StructuralRepeal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockAmendment {
    pub position: Reference,
    pub pure_insertion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextAmendment {
    pub reference: TextAmendmentReference,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextAmendmentReference {
    SAE {
        reference: Reference,
        #[serde(default, skip_serializing_if = "TextAmendmentSAEPart::is_default")]
        amended_part: TextAmendmentSAEPart,
    },
    Structural(StructuralReference),
    ArticleTitle(Reference),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextAmendmentSAEPart {
    #[default]
    All,
    IntroOnly,
    WrapUpOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnforcementDate {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positions: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structural_positions: Vec<StructuralReference>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_default: bool,
    pub date: EnforcementDateType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_repeal: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnforcementDateType {
    Date(NaiveDate),
    DaysAfterPublication(u16),
    DayInMonthAfterPublication {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        month: Option<u8>,
        day: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repeal {
    pub positions: Vec<Reference>,
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

impl TextAmendmentSAEPart {
    pub fn is_default(&self) -> bool {
        *self == TextAmendmentSAEPart::All
    }
}
