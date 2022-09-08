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

use crate::structure::{
    Act, AlphabeticPoint, AlphabeticSubpoint, Article, NumericPoint, NumericSubpoint, Paragraph,
};

use super::{parts::RefPartFrom, Reference};

pub trait ReferenceToElement {
    fn reference(&self) -> Reference;
}

impl ReferenceToElement for Act {
    fn reference(&self) -> Reference {
        Reference {
            act: Some(self.identifier),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for Article {
    fn reference(&self) -> Reference {
        Reference {
            article: Some(RefPartFrom::from_single(self.identifier)),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for Paragraph {
    fn reference(&self) -> Reference {
        // NOTE: this returns an empty reference when identifier = None
        //       this shouldn't be a problem though, as this function is meant
        //       to be used with relative_to(), and that handles empty refs OK.
        Reference {
            paragraph: self.identifier.map(RefPartFrom::from_single),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for AlphabeticPoint {
    fn reference(&self) -> Reference {
        Reference {
            point: Some(RefPartFrom::from_single(self.identifier)),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for NumericPoint {
    fn reference(&self) -> Reference {
        Reference {
            point: Some(RefPartFrom::from_single(self.identifier)),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for AlphabeticSubpoint {
    fn reference(&self) -> Reference {
        Reference {
            subpoint: Some(RefPartFrom::from_single(self.identifier)),
            ..Default::default()
        }
    }
}

impl ReferenceToElement for NumericSubpoint {
    fn reference(&self) -> Reference {
        Reference {
            subpoint: Some(RefPartFrom::from_single(self.identifier)),
            ..Default::default()
        }
    }
}
