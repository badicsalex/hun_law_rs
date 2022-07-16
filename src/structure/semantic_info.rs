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

use serde::{Deserialize, Serialize};

use super::ActIdentifier;
use crate::reference::Reference;
use crate::util::is_default;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticInfo {
    #[serde(default, skip_serializing_if = "is_default")]
    pub outgoing_references: Vec<OutgoingReference>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub new_abbreviations: Vec<ActIdAbbreviation>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub special_phrase: Option<SpecialPhrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingReference {
    pub start: usize,
    pub end: usize,
    pub reference: Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActIdAbbreviation {
    pub act_id: ActIdentifier,
    pub abbreviation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialPhrase {
    ArticleTitleAmendment,
    BlockAmendment,
    EnforcementDate,
    Repeal,
    TextAmendment,
}
