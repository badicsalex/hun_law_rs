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

pub mod builder;
pub mod parts;
pub mod structural;
pub mod to_element;
pub mod unchecked;

pub mod string;
#[cfg(test)]
mod tests;

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};

use self::{
    builder::ReferenceBuilder,
    parts::{AnyReferencePart, RefPartArticle, RefPartParagraph, RefPartPoint, RefPartSubpoint},
    unchecked::UncheckedReference,
};
use crate::{
    identifier::{
        range::{IdentifierRange, IdentifierRangeFrom},
        ActIdentifier, ArticleIdentifier,
    },
    reference::builder::ReferenceBuilderSetPart,
};

/// Reference to an Act, article or SAE. Possibly relative.
///
/// Guarantees:
/// - There are no 'gaps' in the parts, apart from a potentially missing paragraph
///   (in that case, it means the 'default paragraph' of the article
/// - It might be a range, but the range part is always the last part of the reference
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "UncheckedReference")]
#[serde(into = "UncheckedReference")]
pub struct Reference {
    act: Option<ActIdentifier>,
    article: Option<RefPartArticle>,
    paragraph: Option<RefPartParagraph>,
    point: Option<RefPartPoint>,
    subpoint: Option<RefPartSubpoint>,
}

impl Reference {
    pub fn get_last_part(&self) -> AnyReferencePart {
        self.subpoint
            .map(|x| x.into())
            .or_else(|| self.point.map(|x| x.into()))
            .or_else(|| self.paragraph.map(|x| x.into()))
            .or_else(|| self.article.map(|x| x.into()))
            .or_else(|| self.act.map(|x| x.into()))
            .unwrap_or(AnyReferencePart::Empty)
    }

    pub fn is_act_only(&self) -> bool {
        self.article.is_none()
    }

    pub fn act(&self) -> Option<ActIdentifier> {
        self.act
    }

    pub fn article(&self) -> Option<RefPartArticle> {
        self.article
    }

    pub fn without_act(&self) -> Reference {
        // This is done without going through an UncheckedReference because
        // there is no way to create an invalid Reference like this.
        Self {
            act: None,
            ..self.clone()
        }
    }

    pub fn first_in_range(&self) -> Self {
        Self {
            act: self.act,
            article: self
                .article
                .map(|x| IdentifierRangeFrom::from_single(x.first_in_range())),
            paragraph: self
                .paragraph
                .map(|x| IdentifierRangeFrom::from_single(x.first_in_range())),
            point: self.point.map(|x| match x {
                RefPartPoint::Numeric(n) => IdentifierRangeFrom::from_single(n.first_in_range()),
                RefPartPoint::Alphabetic(a) => IdentifierRangeFrom::from_single(a.first_in_range()),
            }),
            subpoint: self.subpoint.map(|x| match x {
                RefPartSubpoint::Numeric(n) => IdentifierRangeFrom::from_single(n.first_in_range()),
                RefPartSubpoint::Alphabetic(a) => {
                    IdentifierRangeFrom::from_single(a.first_in_range())
                }
            }),
        }
    }

    pub fn last_in_range(&self) -> Self {
        Self {
            act: self.act,
            article: self
                .article
                .map(|x| IdentifierRangeFrom::from_single(x.last_in_range())),
            paragraph: self
                .paragraph
                .map(|x| IdentifierRangeFrom::from_single(x.last_in_range())),
            point: self.point.map(|x| match x {
                RefPartPoint::Numeric(n) => IdentifierRangeFrom::from_single(n.last_in_range()),
                RefPartPoint::Alphabetic(a) => IdentifierRangeFrom::from_single(a.last_in_range()),
            }),
            subpoint: self.subpoint.map(|x| match x {
                RefPartSubpoint::Numeric(n) => IdentifierRangeFrom::from_single(n.last_in_range()),
                RefPartSubpoint::Alphabetic(a) => {
                    IdentifierRangeFrom::from_single(a.last_in_range())
                }
            }),
        }
    }

    pub fn make_range(start: &Self, end: &Self) -> Result<Self> {
        let mut builder = ReferenceBuilder::new();
        ensure!(
            start.act == end.act,
            "Reference ranges between acts are not allowed"
        );
        if let Some(act) = start.act {
            builder.set_part(act);
        }

        // --- article ---
        if start.article != end.article {
            ensure!(
                start.paragraph.is_none()
                    && end.paragraph.is_none()
                    && start.point.is_none()
                    && end.point.is_none()
                    && start.subpoint.is_none()
                    && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (article)"
            );
            if let (Some(start_article), Some(end_article)) = (&start.article, &end.article) {
                builder.set_part(RefPartArticle::from_range(
                    start_article.first_in_range(),
                    end_article.last_in_range(),
                ));
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (article)")
        }

        if let Some(article) = &start.article {
            builder.set_part(*article);
        }

        // --- paragraph ---
        if start.paragraph != end.paragraph {
            ensure!(
                start.point.is_none()
                    && end.point.is_none()
                    && start.subpoint.is_none()
                    && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (paragraph)"
            );

            if let (Some(start_paragraph), Some(end_paragraph)) = (&start.paragraph, &end.paragraph)
            {
                builder.set_part(RefPartParagraph::from_range(
                    start_paragraph.first_in_range(),
                    end_paragraph.last_in_range(),
                ));
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (paragraph)")
        }

        if let Some(paragraph) = &start.paragraph {
            builder.set_part(*paragraph);
        }

        // --- point ---
        if start.point != end.point {
            ensure!(
                start.subpoint.is_none() && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (point)"
            );
            if let (Some(start_point), Some(end_point)) = (&start.point, &end.point) {
                match (start_point, end_point) {
                    (RefPartPoint::Numeric(sp), RefPartPoint::Numeric(ep)) => builder.set_part(
                        RefPartPoint::from_range(sp.first_in_range(), ep.last_in_range()),
                    ),
                    (RefPartPoint::Alphabetic(sp), RefPartPoint::Alphabetic(ep)) => builder
                        .set_part(RefPartPoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    _ => bail!("Point id types are different when creating a range."),
                };
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (point)")
        }

        if let Some(point) = &start.point {
            builder.set_part(*point);
        }

        // --- subpoint ---
        if start.subpoint != end.subpoint {
            if let (Some(start_subpoint), Some(end_subpoint)) = (&start.subpoint, &end.subpoint) {
                match (start_subpoint, end_subpoint) {
                    (RefPartSubpoint::Numeric(sp), RefPartSubpoint::Numeric(ep)) => builder
                        .set_part(RefPartSubpoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    (RefPartSubpoint::Alphabetic(sp), RefPartSubpoint::Alphabetic(ep)) => builder
                        .set_part(RefPartSubpoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    _ => bail!("subpoint id types are different when creating a range."),
                };
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (subpoint)")
        }

        if let Some(subpoint) = &start.subpoint {
            builder.set_part(*subpoint);
        }
        builder.build()
    }

    pub fn is_parent_of(&self, other: &Reference) -> bool {
        if self.act != other.act {
            false
        } else if self.article != other.article {
            self.article.is_none()
        } else if self.paragraph != other.paragraph {
            self.paragraph.is_none()
        } else if self.point != other.point {
            self.point.is_none()
        } else if self.subpoint != other.subpoint {
            self.subpoint.is_none()
        } else {
            false
        }
    }

    pub fn contains(&self, other: &Reference) -> bool {
        let self_first = self.first_in_range();
        let self_last = self.last_in_range();
        let other_first = other.first_in_range();
        let other_last = other.last_in_range();
        ((self_first <= other_first) || (self_first.is_parent_of(&other_first)))
            && ((self_last >= other_last) || (self_last.is_parent_of(&other_last)))
    }

    pub fn parent(&self) -> Reference {
        if self.subpoint.is_some() {
            Self {
                subpoint: None,
                ..self.clone()
            }
        } else if self.point.is_some() {
            Self {
                point: None,
                ..self.clone()
            }
        } else if self.paragraph.is_some() {
            Self {
                paragraph: None,
                ..self.clone()
            }
        } else if self.article.is_some() {
            Self {
                article: None,
                ..self.clone()
            }
        } else {
            Default::default()
        }
    }

    pub fn relative_to(&self, other: &Reference) -> Result<Reference> {
        let result: UncheckedReference = if self.act.is_some() {
            self.into()
        } else if self.article.is_some() {
            UncheckedReference {
                act: other.act,
                ..self.into()
            }
        } else if self.paragraph.is_some() {
            UncheckedReference {
                act: other.act,
                article: other.article,
                ..self.into()
            }
        } else if self.point.is_some() {
            UncheckedReference {
                act: other.act,
                article: other.article,
                paragraph: other.paragraph,
                ..self.into()
            }
        } else if self.subpoint.is_some() {
            UncheckedReference {
                act: other.act,
                article: other.article,
                paragraph: other.paragraph,
                point: other.point,
                ..self.into()
            }
        } else {
            other.into()
        };
        result.try_into()
    }
}

impl From<ActIdentifier> for Reference {
    fn from(act: ActIdentifier) -> Self {
        Reference {
            act: Some(act),
            ..Default::default()
        }
    }
}

impl From<(ActIdentifier, IdentifierRange<ArticleIdentifier>)> for Reference {
    fn from((act, article): (ActIdentifier, IdentifierRange<ArticleIdentifier>)) -> Self {
        Reference {
            act: Some(act),
            article: Some(article),
            ..Default::default()
        }
    }
}
