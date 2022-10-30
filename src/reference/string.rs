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

use std::fmt::{Debug, Display, Write};

use anyhow::{anyhow, bail, Result};

use super::{
    parts::{RefPartPoint, RefPartSubpoint},
    unchecked::UncheckedReference,
    Reference,
};
use crate::util::compact_string::CompactString;

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(act) = self.act {
            write!(f, "{}", act)?;
            if self.article.is_some() {
                f.write_char(' ')?
            }
        }
        if let Some(article) = self.article {
            if !article.is_range() {
                write!(f, "{}.", article.first_in_range())?;
            } else {
                write!(
                    f,
                    "{}–{}.",
                    article.first_in_range(),
                    article.last_in_range()
                )?;
            }
            if self.paragraph.is_some() || self.point.is_some() {
                f.write_str(" § ")?;
            } else {
                f.write_str(" §-a")?;
            }
        }
        if let Some(paragraph) = self.paragraph {
            if !paragraph.is_range() {
                write!(f, "({})", paragraph.first_in_range())?;
            } else {
                write!(
                    f,
                    "({})–({})",
                    paragraph.first_in_range(),
                    paragraph.last_in_range()
                )?;
            }
            if self.point.is_some() {
                f.write_str(" bekezdés ")?;
            } else {
                f.write_str(" bekezdése")?;
            }
        }
        if let Some(point) = self.point {
            match point {
                RefPartPoint::Numeric(point) => {
                    if !point.is_range() {
                        write!(f, "{}.", point.first_in_range())?;
                    } else {
                        write!(f, "{}–{}.", point.first_in_range(), point.last_in_range())?;
                    }
                }
                RefPartPoint::Alphabetic(point) => {
                    if !point.is_range() {
                        write!(f, "{})", point.first_in_range())?;
                    } else {
                        write!(f, "{})–{})", point.first_in_range(), point.last_in_range())?;
                    }
                }
            }
            if self.subpoint.is_some() {
                f.write_str(" pont ")?;
            } else {
                f.write_str(" pontja")?;
            }
        }
        if let Some(subpoint) = self.subpoint {
            match subpoint {
                RefPartSubpoint::Numeric(subpoint) => {
                    if !subpoint.is_range() {
                        write!(f, "{}.", subpoint.first_in_range())?;
                    } else {
                        write!(
                            f,
                            "{}–{}.",
                            subpoint.first_in_range(),
                            subpoint.last_in_range()
                        )?;
                    }
                }
                RefPartSubpoint::Alphabetic(subpoint) => {
                    if !subpoint.is_range() {
                        write!(f, "{})", subpoint.first_in_range())?;
                    } else {
                        write!(
                            f,
                            "{})–{})",
                            subpoint.first_in_range(),
                            subpoint.last_in_range()
                        )?;
                    }
                }
            }
            f.write_str(" alpontja")?;
        }
        Ok(())
    }
}

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

    use crate::identifier::{
        range::IdentifierRangeFrom, ActIdentifier, AlphabeticIdentifier, IdentifierCommon,
        NumericIdentifier, PrefixedAlphabeticIdentifier,
    };

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
    fn test_display_string() {
        let mut base = Reference {
            act: Some(ActIdentifier {
                year: 2012,
                number: 1,
            }),
            article: quick_convert_part("1"),
            paragraph: quick_convert_part("2"),
            point: quick_convert_part::<RefPartPoint, NumericIdentifier>("3"),
            subpoint: quick_convert_part::<RefPartSubpoint, PrefixedAlphabeticIdentifier>("a"),
        };
        assert_eq!(
            &format!("{}", base),
            "2012. évi I. törvény 1. § (2) bekezdés 3. pont a) alpontja"
        );
        assert_eq!(
            &format!("{}", base.parent()),
            "2012. évi I. törvény 1. § (2) bekezdés 3. pontja"
        );
        assert_eq!(
            &format!("{}", base.parent().parent()),
            "2012. évi I. törvény 1. § (2) bekezdése"
        );
        assert_eq!(
            &format!("{}", base.parent().parent().parent()),
            "2012. évi I. törvény 1. §-a"
        );
        assert_eq!(
            &format!("{}", base.parent().parent().parent().parent()),
            "2012. évi I. törvény"
        );

        base.article = quick_convert_part("1:123/B");
        base.point = quick_convert_part::<RefPartPoint, AlphabeticIdentifier>("a");
        base.subpoint = quick_convert_part::<RefPartSubpoint, NumericIdentifier>("2/a");
        assert_eq!(
            &format!("{}", base),
            "2012. évi I. törvény 1:123/B. § (2) bekezdés a) pont 2a. alpontja"
        );
        assert_eq!(
            &format!("{}", base.parent()),
            "2012. évi I. törvény 1:123/B. § (2) bekezdés a) pontja"
        );

        let some_missing = Reference {
            article: quick_convert_part("1"),
            point: quick_convert_part::<RefPartPoint, AlphabeticIdentifier>("a-x"),
            ..Default::default()
        };

        assert_eq!(&format!("{}", some_missing), "1. § a)–x) pontja");

        assert_eq!(&format!("{}", Reference::default()), "");
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
