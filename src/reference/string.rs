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

use std::fmt::{Debug, Write};

use anyhow::{anyhow, bail, Result};

use super::{unchecked::UncheckedReference, Reference};
use crate::util::compact_string::CompactString;

impl Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Reference");
        self.act.map(|act| debug_struct.field("act", &act));
        self.article
            .map(|article| debug_struct.field("article", &article));
        self.paragraph
            .map(|paragraph| debug_struct.field("paragraph", &paragraph));
        self.point.map(|point| debug_struct.field("point", &point));
        self.subpoint
            .map(|subpoint| debug_struct.field("subpoint", &subpoint));
        debug_struct.finish()
    }
}

impl CompactString for Reference {
    fn fmt_compact_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.act.fmt_compact_string(f)?;
        f.write_char('_')?;
        self.article.fmt_compact_string(f)?;
        f.write_char('_')?;
        self.paragraph.fmt_compact_string(f)?;
        f.write_char('_')?;
        self.point.fmt_compact_string(f)?;
        f.write_char('_')?;
        self.subpoint.fmt_compact_string(f)?;
        Ok(())
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        let mut iter = s.as_ref().split('_');
        let act = iter
            .next()
            .ok_or_else(|| anyhow!("Not enough parts in Reference::from_compact_string"))?;
        let article = iter
            .next()
            .ok_or_else(|| anyhow!("Not enough parts in Reference::from_compact_string"))?;
        let paragraph = iter
            .next()
            .ok_or_else(|| anyhow!("Not enough parts in Reference::from_compact_string"))?;
        let point = iter
            .next()
            .ok_or_else(|| anyhow!("Not enough parts in Reference::from_compact_string"))?;
        let subpoint = iter
            .next()
            .ok_or_else(|| anyhow!("Not enough parts in Reference::from_compact_string"))?;
        if iter.next().is_some() {
            bail!("Too many parts in Reference::from_compact_string")
        }
        let result = UncheckedReference {
            act: CompactString::from_compact_string(act)?,
            article: CompactString::from_compact_string(article)?,
            paragraph: CompactString::from_compact_string(paragraph)?,
            point: CompactString::from_compact_string(point)?,
            subpoint: CompactString::from_compact_string(subpoint)?,
        };
        result.try_into()
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use crate::{identifier::{
        range::IdentifierRangeFrom, ActIdentifier, AlphabeticIdentifier, IdentifierCommon,
        NumericIdentifier,
    }, reference::parts::{RefPartPoint, RefPartSubpoint}};

    use super::*;
    fn quick_convert_part<TR, TI>(s: &str) -> Option<TR>
    where
        TR: IdentifierRangeFrom<TI>,
        TI: IdentifierCommon + FromStr,
        <TI as FromStr>::Err: Debug,
    {
        if s.is_empty() {
            None
        } else if let Some((start, end)) = s.split_once('-') {
            Some(TR::from_range(start.parse().unwrap(), end.parse().unwrap()))
        } else {
            Some(TR::from_single(s.parse().unwrap()))
        }
    }

    #[test]
    fn test_compact_string() {
        let base = Reference {
            act: Some(ActIdentifier {
                year: 2012,
                number: 1,
            }),
            article: quick_convert_part("1"),
            paragraph: quick_convert_part("2"),
            point: quick_convert_part::<RefPartPoint, NumericIdentifier>("3"),
            subpoint: quick_convert_part::<RefPartSubpoint, NumericIdentifier>("4"),
        };
        assert_eq!(&base.compact_string().to_string(), "2012.1_1_2_3_4");
        assert_eq!(
            base,
            Reference::from_compact_string("2012.1_1_2_3_4").unwrap()
        );

        let some_missing = Reference {
            article: quick_convert_part("1"),
            point: quick_convert_part::<RefPartPoint, AlphabeticIdentifier>("a-x"),
            ..Default::default()
        };

        assert_eq!(&some_missing.compact_string().to_string(), "_1__a-x_");
        assert_eq!(
            some_missing,
            Reference::from_compact_string("_1__a-x_").unwrap()
        );

        assert_eq!(&Reference::default().compact_string().to_string(), "____");
        assert_eq!(
            Reference::default(),
            Reference::from_compact_string("____").unwrap()
        );

        assert!(Reference::from_compact_string("_1__a-x").is_err());
        assert!(Reference::from_compact_string("_1__a-x__").is_err());
        assert!(Reference::from_compact_string("_a__a-x__").is_err());
    }
}
