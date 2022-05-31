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

use std::ops::RangeInclusive;

pub type ActReference = ReferencePart<ArticleReference>;
pub type ArticleReference = ReferencePart<ParagraphReference>;
pub type ParagraphReference = ReferencePart<PointReference>;
pub type PointReference = ReferencePart<SubpointReference>;
pub type SubpointReference = ReferencePart<NoChildren>;

#[derive(Debug, Clone)]
pub enum ReferencePart<ChildType> {
    Single {
        identifier: String,
    },
    Range(RangeInclusive<String>),
    WithChild {
        identifier: String,
        child: ChildType,
    },
}

#[derive(Debug, Clone)]
pub enum NoChildren {}

trait ChildOf<T1> {}

trait RelativeTo<Other> {
    fn relative_to(&self, other: &Other) -> Other;
}

trait FromIdAndChild<ChildType> {
    fn from_id_and_child(identifier: String, child: ChildType) -> Self;
}

impl<ChildType> FromIdAndChild<ChildType> for ReferencePart<ChildType> {
    fn from_id_and_child(identifier: String, child: ChildType) -> Self {
        Self::WithChild { identifier, child }
    }
}

impl<T: Clone> RelativeTo<ReferencePart<ReferencePart<T>>> for ReferencePart<T> {
    fn relative_to(
        &self,
        other: &ReferencePart<ReferencePart<T>>,
    ) -> ReferencePart<ReferencePart<T>> {
        match other {
            ReferencePart::Single { identifier } => ReferencePart::WithChild {
                identifier: identifier.clone(),
                child: (*self).clone(),
            },

            ReferencePart::Range(_) => panic!(),
            ReferencePart::WithChild { identifier, .. } => ReferencePart::WithChild {
                identifier: identifier.clone(),
                child: (*self).clone(),
            },
        }
    }
}

impl<T: Clone> RelativeTo<ReferencePart<ReferencePart<ReferencePart<T>>>> for ReferencePart<T> {
    fn relative_to(
        &self,
        other: &ReferencePart<ReferencePart<ReferencePart<T>>>,
    ) -> ReferencePart<ReferencePart<ReferencePart<T>>> {
        match other {
            ReferencePart::Single {.. } => panic!(),
            ReferencePart::Range(_) => panic!(),
            ReferencePart::WithChild { identifier, child } => ReferencePart::WithChild {
                identifier: identifier.clone(),
                child: self.relative_to(child),
            },
        }
    }
}

impl<T: Clone> RelativeTo<ReferencePart<ReferencePart<ReferencePart<ReferencePart<T>>>>> for ReferencePart<T> {
    fn relative_to(
        &self,
        other: &ReferencePart<ReferencePart<ReferencePart<ReferencePart<T>>>>,
    ) -> ReferencePart<ReferencePart<ReferencePart<ReferencePart<T>>>> {
        match other {
            ReferencePart::Single {.. } => panic!(),
            ReferencePart::Range(_) => panic!(),
            ReferencePart::WithChild { identifier, child } => ReferencePart::WithChild {
                identifier: identifier.clone(),
                child: self.relative_to(child),
            },
        }
    }
}

fn tinytest() {
    let a = ArticleReference::Single {
        identifier: "lel".to_string(),
    };
    let b = SubpointReference::Single {
        identifier: "lel".to_string(),
    };
    b.relative_to(&a);
}
