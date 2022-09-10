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

use anyhow::{anyhow, ensure, Result};

use serde::{Deserialize, Serialize};

use crate::identifier::ActIdentifier;

use super::{
    parts::{RefPartArticle, RefPartParagraph, RefPartPoint, RefPartSubpoint},
    Reference,
};

/// Helper to create Reference instances from parts.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UncheckedReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) act: Option<ActIdentifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) article: Option<RefPartArticle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) paragraph: Option<RefPartParagraph>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) point: Option<RefPartPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) subpoint: Option<RefPartSubpoint>,
}

impl UncheckedReference {
    fn check_part_combination(&self) -> Result<()> {
        let filled = [
            self.act.is_some(),
            self.article.is_some(),
            self.paragraph.is_some(),
            self.point.is_some(),
            self.subpoint.is_some(),
        ];
        match filled {
            // Just act ref or empty ref
            [_, false, false, false, false] => Ok(()),
            // Article, maybe point, no subpoint. Paragraph can be missing
            [_, true, _, _, false] => Ok(()),
            // Subpoint
            [_, true, _, true, true] => Ok(()),
            // Relative paragraph
            [false, false, true, false, false] => Ok(()),
            // Relative point or subpoint.
            [false, false, _, true, _] => Ok(()),
            // Just subpoint. In this case paragraph is not allowed, as point would be a gap
            [false, false, false, false, true] => Ok(()),
            _ => Err(anyhow!("Invalid reference part combination: {:?}", self)),
        }
    }

    fn check_ranges(&self) -> Result<()> {
        if let Some(article) = &self.article {
            if article.is_range() {
                ensure!(
                    self.paragraph.is_none() && self.point.is_none() && self.subpoint.is_none(),
                    "Reference parts found after article range"
                );
            }
        }
        if let Some(paragraph) = &self.paragraph {
            if paragraph.is_range() {
                ensure!(
                    self.point.is_none() && self.subpoint.is_none(),
                    "Reference parts found after paragraph range"
                );
            }
        }
        if let Some(point) = &self.point {
            if point.is_range() {
                ensure!(
                    self.subpoint.is_none(),
                    "Reference parts found after point range"
                );
            }
        }
        Ok(())
    }
}

impl From<Reference> for UncheckedReference {
    fn from(r: Reference) -> Self {
        Self {
            act: r.act,
            article: r.article,
            paragraph: r.paragraph,
            point: r.point,
            subpoint: r.subpoint,
        }
    }
}

impl From<&Reference> for UncheckedReference {
    fn from(r: &Reference) -> Self {
        r.clone().into()
    }
}

impl TryFrom<UncheckedReference> for Reference {
    type Error = anyhow::Error;

    fn try_from(r: UncheckedReference) -> Result<Self, Self::Error> {
        r.check_part_combination()?;
        r.check_ranges()?;
        Ok(Self {
            act: r.act,
            article: r.article,
            paragraph: r.paragraph,
            point: r.point,
            subpoint: r.subpoint,
        })
    }
}
