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

use anyhow::{anyhow, Result};

use super::{
    abbreviation::AbbreviationCache,
    reference::{FeedReferenceBuilder, OutgoingReferenceBuilder},
};
use crate::{
    structure::semantic_info::{self, EnforcementDateType},
    util::{
        date::{self, text_to_month_hun},
        str_to_int_hun,
    },
};
use hun_law_grammar::*;

pub fn convert_enforcement_date(
    abbreviation_cache: &AbbreviationCache,
    elem: &EnforcementDate,
) -> Result<semantic_info::EnforcementDate> {
    let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
    ref_builder.feed(&elem.references)?;
    let positions = ref_builder
        .get_result()
        .into_iter()
        .map(From::from)
        .collect();
    let date = (&elem.date).try_into()?;
    let inline_repeal = if let Some(ir) = &elem.inline_repeal {
        Some(ir.try_into()?)
    } else {
        None
    };

    Ok(semantic_info::EnforcementDate {
        positions,
        date,
        inline_repeal,
    })
}

impl TryFrom<&EnforcementDate_date> for semantic_info::EnforcementDateType {
    type Error = anyhow::Error;

    fn try_from(value: &EnforcementDate_date) -> Result<Self, Self::Error> {
        match value {
            EnforcementDate_date::AfterPublication(x) => x.try_into(),
            EnforcementDate_date::Date(x) => x.try_into(),
            EnforcementDate_date::DayInMonth(x) => x.try_into(),
        }
    }
}

impl TryFrom<&AfterPublication> for EnforcementDateType {
    type Error = anyhow::Error;

    fn try_from(value: &AfterPublication) -> Result<Self, Self::Error> {
        Ok(EnforcementDateType::DaysAfterPublication(
            if let Some(x) = &value.days {
                match x {
                    AfterPublication_days::HungarianOrdinal(num) => {
                        str_to_int_hun(num).ok_or_else(|| anyhow!("Invalid hungarian ordinal"))?
                    }
                    AfterPublication_days::NumberWithDot(num) => num.parse()?,
                }
            } else {
                1
            },
        ))
    }
}

impl TryFrom<&Date> for EnforcementDateType {
    type Error = anyhow::Error;

    fn try_from(value: &Date) -> Result<Self, Self::Error> {
        Ok(EnforcementDateType::Date(value.try_into()?))
    }
}

impl TryFrom<&Date> for date::Date {
    type Error = anyhow::Error;

    fn try_from(value: &Date) -> Result<Self, Self::Error> {
        Ok(date::Date {
            year: value.year.parse()?,
            month: text_to_month_hun(&value.month)?,
            day: value.day.parse()?,
        })
    }
}

impl TryFrom<&DayInMonth> for EnforcementDateType {
    type Error = anyhow::Error;

    fn try_from(value: &DayInMonth) -> Result<Self, Self::Error> {
        let month = if let Some(m) = &value.month {
            Some(
                str_to_int_hun(m)
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or_else(|| anyhow!("Invalid hungarian ordinal"))?,
            )
        } else {
            None
        };
        let day = match &value.day {
            DayInMonth_day::HungarianOrdinal(num) => {
                str_to_int_hun(num).ok_or_else(|| anyhow!("Invalid hungarian ordinal"))?
            }
            DayInMonth_day::NumberWithDot(num) => num.parse()?,
        };
        Ok(Self::DayInMonthAfterPublication { month, day })
    }
}
