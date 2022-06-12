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

pub struct Reference<T1, T2, T3, T4, T5> {
    act: T1,
    article: T2,
    paragraph: T3,
    point: T4,
    subpoint: T5,
    _dont_construct: (),
}

pub trait RefPart: Clone {}

impl<T2: RefPart, T3: RefPart, T4: RefPart, T5: RefPart> Reference<(), T2, T3, T4, T5> {
    pub fn relative_to<W1, W2, W3, W4, W5>(
        self,
        other: Reference<W1, W2, W3, W4, W5>,
    ) -> Reference<W1, T2, T3, T4, T5> {
        Reference {
            act: other.act,
            article: self.article,
            paragraph: self.paragraph,
            point: self.point,
            subpoint: self.subpoint,
            _dont_construct: (),
        }
    }
}

impl<T3: RefPart, T4: RefPart, T5: RefPart> Reference<(), (), T3, T4, T5> {
    pub fn relative_to<W1, W2, W3, W4, W5>(
        self,
        other: Reference<W1, W2, W3, W4, W5>,
    ) -> Reference<W1, W2, T3, T4, T5> {
        Reference {
            act: other.act,
            article: other.article,
            paragraph: self.paragraph,
            point: self.point,
            subpoint: self.subpoint,
            _dont_construct: (),
        }
    }
}

pub type RefRangePart = RangeInclusive<String>;

pub enum AnyReference {
    Empty(Reference<(), (), (), (), ()>),
    Act(Reference<String, (), (), (), ()>),

    ActArticle(Reference<String, String, (), (), ()>),
    ActParagraph(Reference<String, String, String, (), ()>),
    ActPoint(Reference<String, String, String, String, ()>),
    ActSubpoint(Reference<String, String, String, String, String>),

    Article(Reference<(), String, (), (), ()>),
    ArticleParagraph(Reference<(), String, String, (), ()>),
    ArticlePoint(Reference<(), String, String, String, ()>),
    ArticleSubpoint(Reference<(), String, String, String, String>),

    Paragraph(Reference<(), (), String, (), ()>),
    ParagraphPoint(Reference<(), (), String, String, ()>),
    ParagraphSubpoint(Reference<(), (), String, String, String>),

    Point(Reference<(), (), (), String, ()>),
    PointSubpoint(Reference<(), (), (), String, String>),

    Subpoint(Reference<(), (), (), (), String>),

    ActArticleRange(Reference<String, RefRangePart, (), (), ()>),
    ActParagraphRange(Reference<String, String, RefRangePart, (), ()>),
    ActPointRange(Reference<String, String, String, RefRangePart, ()>),
    ActSubpointRange(Reference<String, String, String, String, RefRangePart>),

    ArticleRange(Reference<(), RefRangePart, (), (), ()>),
    ArticleParagraphRange(Reference<(), String, RefRangePart, (), ()>),
    ArticlePointRange(Reference<(), String, String, RefRangePart, ()>),
    ArticleSubpointRange(Reference<(), String, String, String, RefRangePart>),

    ParagraphRange(Reference<(), (), RefRangePart, (), ()>),
    ParagraphPointRange(Reference<(), (), String, RefRangePart, ()>),
    ParagraphSubpointRange(Reference<(), (), String, String, RefRangePart>),

    PointRange(Reference<(), (), (), RefRangePart, ()>),
    PointSubpointRange(Reference<(), (), (), String, RefRangePart>),

    SubpointRange(Reference<(), (), (), (), RefRangePart>),
}
