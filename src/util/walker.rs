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

use crate::{
    identifier::IsNextFrom,
    semantic_info::SemanticInfo,
    structure::{
        Act, ActChild, AlphabeticPointChildren, AlphabeticSubpointChildren, NumericPointChildren,
        NumericSubpointChildren, ParagraphChildren, SAEBody, SubArticleElement,
    },
    util::IsDefault,
};
use anyhow::{Context, Result};

use super::debug::{DebugContextString, WithElemContext};

/// Visit every SAE in the object in a typeless way
/// SAEs in Block Amendments are not traversed into.
pub trait SAEVisitor {
    /// Called on entering a SAE which have children
    fn on_enter(
        &mut self,
        intro: &mut String,
        wrap_up: &mut Option<String>,
        semantic_info: &mut SemanticInfo,
    ) -> Result<()>;

    /// Called on exiting a SAE which have children
    fn on_exit(
        &mut self,
        intro: &mut String,
        wrap_up: &mut Option<String>,
        semantic_info: &mut SemanticInfo,
    ) -> Result<()>;

    /// Called on SAEs which have no children (instead of enter and exit)
    fn on_text(&mut self, text: &mut String, semantic_info: &mut SemanticInfo) -> Result<()>;
}

pub trait WalkSAE {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()>;
}

impl<T: WalkSAE + DebugContextString> WalkSAE for Vec<T> {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        self.iter_mut().try_for_each(|c| {
            c.walk_saes(visitor)
                .with_elem_context("Error walking multiple", c)
        })
    }
}

impl WalkSAE for Act {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        self.children.walk_saes(visitor)
    }
}

impl WalkSAE for ActChild {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        if let ActChild::Article(article) = self {
            article.children.walk_saes(visitor)
        } else {
            Ok(())
        }
    }
}

impl<IT, CT> WalkSAE for SubArticleElement<IT, CT>
where
    Self: DebugContextString,
    CT: WalkSAE,
    IT: IsDefault + IsNextFrom + Clone + std::fmt::Debug + Eq,
{
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        match &mut self.body {
            SAEBody::Text(text) => visitor
                .on_text(text, &mut self.semantic_info)
                .with_context(|| "'on_text' call failed"),

            SAEBody::Children {
                intro,
                children,
                wrap_up,
            } => {
                visitor
                    .on_enter(intro, wrap_up, &mut self.semantic_info)
                    .with_context(|| "'on_enter' call failed")?;
                children.walk_saes(visitor)?;
                visitor
                    .on_exit(intro, wrap_up, &mut self.semantic_info)
                    .with_context(|| "'on_exit' call failed")
            }
        }
    }
}

impl WalkSAE for ParagraphChildren {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        match self {
            ParagraphChildren::AlphabeticPoint(b) => b.walk_saes(visitor),
            ParagraphChildren::NumericPoint(b) => b.walk_saes(visitor),
            ParagraphChildren::QuotedBlock(_) | ParagraphChildren::BlockAmendment(_) => Ok(()),
        }
    }
}

impl WalkSAE for AlphabeticPointChildren {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        match self {
            AlphabeticPointChildren::AlphabeticSubpoint(b) => b.walk_saes(visitor),
            AlphabeticPointChildren::NumericSubpoint(b) => b.walk_saes(visitor),
        }
    }
}

impl WalkSAE for NumericPointChildren {
    fn walk_saes<V: SAEVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        match self {
            NumericPointChildren::AlphabeticSubpoint(b) => b.walk_saes(visitor),
        }
    }
}

impl WalkSAE for AlphabeticSubpointChildren {
    fn walk_saes<V: SAEVisitor>(&mut self, _visitor: &mut V) -> Result<()> {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
}

impl WalkSAE for NumericSubpointChildren {
    fn walk_saes<V: SAEVisitor>(&mut self, _visitor: &mut V) -> Result<()> {
        // This is an empty enum, the function shall never run.
        match *self {}
    }
}
