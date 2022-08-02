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

use anyhow::Result;

use crate::{
    semantic_info::{SemanticInfo, SpecialPhrase},
    structure::{Act, ActChild, Article, BlockAmendment, Paragraph, ParagraphChildren, SAEBody},
};

impl Act {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        for act_child in &mut self.children {
            if let ActChild::Article(article) = act_child {
                article.convert_block_amendments()?
            }
        }
        Ok(())
    }
}

impl Article {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        for paragraph in &mut self.children {
            paragraph.convert_block_amendments()?;
        }
        Ok(())
    }
}

impl Paragraph {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        let children = if let SAEBody::Children { children, .. } = &mut self.body {
            children
        } else {
            return Ok(());
        };

        if let ParagraphChildren::QuotedBlock(qbs) = children {
            if qbs.len() != 1 {
                return Ok(());
            }
            let quoted_block = &mut qbs[0];
            if let Some(SemanticInfo {
                special_phrase: Some(special_phrase),
                ..
            }) = &self.semantic_info
            {
                match special_phrase {
                    SpecialPhrase::BlockAmendment(_)
                    | SpecialPhrase::StructuralBlockAmendment(_) => {
                        // TODO: actual parsing
                        *children = ParagraphChildren::BlockAmendment(BlockAmendment {
                            intro: std::mem::take(&mut quoted_block.intro),
                            children: vec![],
                            wrap_up: std::mem::take(&mut quoted_block.wrap_up),
                        })
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
