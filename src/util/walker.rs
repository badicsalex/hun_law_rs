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
    reference::{to_element::ReferenceToElement, Reference},
    semantic_info::SemanticInfo,
    structure::{
        Act, ActChild, AlphabeticPointChildren, AlphabeticSubpointChildren, NumericPointChildren,
        NumericSubpointChildren, ParagraphChildren, SAEBody, SubArticleElement,
    },
    util::IsDefault,
};
use anyhow::{Context, Result};

use super::debug::{DebugContextString, WithElemContext};

macro_rules! impl_walk_sae {
    ($Trait:ident, $Visitor:ident, $walk_fn: tt, $($ref_type: tt)*) => {
        /// Visit every SAE in the object in a typeless way
        /// SAEs in Block Amendments are not traversed into.
        pub trait $Visitor {
            /// Called on entering a SAE which have children
            fn on_enter(
                &mut self,
                position: &Reference,
                intro: $($ref_type)* String,
                wrap_up: $($ref_type)* Option<String>,
                semantic_info: $($ref_type)* SemanticInfo,
            ) -> Result<()> {
                let _ = (position, intro, wrap_up, semantic_info);
                Ok(())
            }

            /// Called on exiting a SAE which have children
            fn on_exit(
                &mut self,
                position: &Reference,
                intro: $($ref_type)* String,
                wrap_up: $($ref_type)* Option<String>,
                semantic_info: $($ref_type)* SemanticInfo,
            ) -> Result<()> {
                let _ = (position, intro, wrap_up, semantic_info);
                Ok(())
            }

            /// Called on SAEs which have no children (instead of enter and exit)
            fn on_text(
                &mut self,
                position: &Reference,
                text: $($ref_type)* String,
                semantic_info: $($ref_type)* SemanticInfo,
            ) -> Result<()> {
                let _ = (position, text, semantic_info);
                Ok(())
            }
        }
        pub trait $Trait {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()>;
        }

        impl<T: $Trait + DebugContextString> $Trait for Vec<T> {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                self.into_iter().try_for_each(|c| {
                    c.$walk_fn(base, visitor)
                        .with_elem_context("Error walking multiple", c)
                })
            }
        }

        impl Act {
            pub fn $walk_fn<V: $Visitor>($($ref_type)* self, visitor: &mut V) -> Result<()> {
                self.children.$walk_fn(&self.reference(), visitor)
            }
        }

        impl $Trait for ActChild {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                if let ActChild::Article(article) = self {
                    article.children.$walk_fn(&article.reference().relative_to(base)?, visitor)
                } else {
                    Ok(())
                }
            }
        }

        impl<IT, CT> $Trait for SubArticleElement<IT, CT>
        where
            Self: DebugContextString + ReferenceToElement,
            CT: $Trait,
            IT: IsDefault + IsNextFrom + Clone + std::fmt::Debug + Eq,
        {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                let element_ref = self.reference().relative_to(base)?;
                match $($ref_type)* self.body {
                    SAEBody::Text(text) => visitor
                        .on_text(&element_ref, text, $($ref_type)* self.semantic_info)
                        .with_context(|| "'on_text' call failed"),

                    SAEBody::Children {
                        intro,
                        children,
                        wrap_up,
                    } => {
                        visitor
                            .on_enter(&element_ref, intro, wrap_up, $($ref_type)* self.semantic_info)
                            .with_context(|| "'on_enter' call failed")?;
                        children.$walk_fn(&element_ref, visitor)?;
                        visitor
                            .on_exit(&element_ref, intro, wrap_up, $($ref_type)* self.semantic_info)
                            .with_context(|| "'on_exit' call failed")
                    }
                }
            }
        }

        impl $Trait for ParagraphChildren {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                match self {
                    ParagraphChildren::AlphabeticPoint(b) => b.$walk_fn(base, visitor),
                    ParagraphChildren::NumericPoint(b) => b.$walk_fn(base, visitor),
                    ParagraphChildren::QuotedBlock(_) | ParagraphChildren::BlockAmendment(_) => {
                        Ok(())
                    }
                }
            }
        }

        impl $Trait for AlphabeticPointChildren {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                match self {
                    AlphabeticPointChildren::AlphabeticSubpoint(b) => b.$walk_fn(base, visitor),
                    AlphabeticPointChildren::NumericSubpoint(b) => b.$walk_fn(base, visitor),
                }
            }
        }

        impl $Trait for NumericPointChildren {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, base: &Reference, visitor: &mut V) -> Result<()> {
                match self {
                    NumericPointChildren::AlphabeticSubpoint(b) => b.$walk_fn(base, visitor),
                }
            }
        }

        impl $Trait for AlphabeticSubpointChildren {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, _base: &Reference, _visitor: &mut V) -> Result<()> {
                // This is an empty enum, the function shall never run.
                match *self {}
            }
        }

        impl $Trait for NumericSubpointChildren {
            fn $walk_fn<V: $Visitor>($($ref_type)* self, _base: &Reference, _visitor: &mut V) -> Result<()> {
                // This is an empty enum, the function shall never run.
                match *self {}
            }
        }
    };
}

impl_walk_sae!(WalkSAE, SAEVisitor, walk_saes, &);
impl_walk_sae!(WalkSAEMut, SAEVisitorMut, walk_saes_mut, &mut);

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{structure::*, util::singleton_yaml};

    const TEST_ACT: &str = r#"
        identifier:
          year: 2345
          number: 13
        subject: A tesztelésről
        preamble: A tesztelés nagyon fontos, és egyben kötelező
        publication_date: 2345-06-07
        children:
        - StructuralElement:
            identifier: '1'
            title: Egyszerű dolgok
            element_type: Book
        - Subtitle:
            title: Alcim id nelkul
        - Article:
            identifier: 1:1
            title: Az egyetlen cikk, aminek cime van.
            children:
            - body: Meg szövege
        - Article:
            identifier: 1:2
            children:
            - identifier: '1'
              body: Valami valami 
            - identifier: '2'
              body:
                intro: Egy felsorolás legyen
                children:
                  AlphabeticPoint:
                  - identifier: a
                    body: többelemű
                  - identifier: b
                    body:
                      intro: kellően
                      children:
                        AlphabeticSubpoint:
                        - identifier: ba
                          body: átláthatatlan
                        - identifier: bb
                          body: komplex
                wrap_up: minden esetben.
        "#;
    #[derive(Debug, Default)]
    struct TestVisitor {
        events: Vec<String>,
    }

    impl SAEVisitor for TestVisitor {
        fn on_text(
            &mut self,
            position: &Reference,
            text: &String,
            _semantic_info: &SemanticInfo,
        ) -> Result<()> {
            self.events.push(format!(
                "TEXT@{}: {}",
                serde_json::to_string(position).unwrap(),
                text
            ));
            Ok(())
        }

        fn on_enter(
            &mut self,
            position: &Reference,
            intro: &String,
            _wrap_up: &Option<String>,
            _semantic_info: &SemanticInfo,
        ) -> Result<()> {
            self.events.push(format!(
                "ENTER@{}: {}",
                serde_json::to_string(position).unwrap(),
                intro
            ));
            Ok(())
        }

        fn on_exit(
            &mut self,
            position: &Reference,
            _intro: &String,
            wrap_up: &Option<String>,
            _semantic_info: &SemanticInfo,
        ) -> Result<()> {
            self.events.push(format!(
                "EXIT@{}: {}",
                serde_json::to_string(position).unwrap(),
                wrap_up.as_ref().map(|s| s as &str).unwrap_or("")
            ));
            Ok(())
        }
    }

    #[test]
    fn test_immutable() {
        let act: Act = singleton_yaml::from_str(TEST_ACT).unwrap();
        let mut visitor = TestVisitor::default();
        act.walk_saes(&mut visitor).unwrap();
        assert_eq!(
            visitor.events.iter().map(|s| s as &str).collect::<Vec<_>>(),
            vec![
                r#"TEXT@{"act":{"year":2345,"number":13},"article":"1:1"}: Meg szövege"#,
                r#"TEXT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"1"}: Valami valami"#,
                r#"ENTER@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2"}: Egy felsorolás legyen"#,
                r#"TEXT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2","point":"a"}: többelemű"#,
                r#"ENTER@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2","point":"b"}: kellően"#,
                r#"TEXT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2","point":"b","subpoint":"ba"}: átláthatatlan"#,
                r#"TEXT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2","point":"b","subpoint":"bb"}: komplex"#,
                r#"EXIT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2","point":"b"}: "#,
                r#"EXIT@{"act":{"year":2345,"number":13},"article":"1:2","paragraph":"2"}: minden esetben."#,
            ]
        );
    }

    const MODIFIED_ACT: &str = r#"
        identifier:
          year: 2345
          number: 13
        subject: A tesztelésről
        preamble: A tesztelés nagyon fontos, és egyben kötelező
        publication_date: 2345-06-07
        children:
        - StructuralElement:
            identifier: '1'
            title: Egyszerű dolgok
            element_type: Book
        - Subtitle:
            title: Alcim id nelkul
        - Article:
            identifier: 1:1
            title: Az egyetlen cikk, aminek cime van.
            children:
            - body: Mag szövaga
        - Article:
            identifier: 1:2
            children:
            - identifier: '1'
              body: Valami valami 
            - identifier: '2'
              body:
                intro: Egy falsorolás lagyan
                children:
                  AlphabeticPoint:
                  - identifier: a
                    body: többalamű
                  - identifier: b
                    body:
                      intro: kallőan
                      children:
                        AlphabeticSubpoint:
                        - identifier: ba
                          body: átláthatatlan
                        - identifier: bb
                          body: komplax
                      wrap_up: wot
        "#;
    #[derive(Debug, Default)]
    struct TestVisitorMut {}

    impl SAEVisitorMut for TestVisitorMut {
        fn on_text(
            &mut self,
            _position: &Reference,
            text: &mut String,
            _semantic_info: &mut SemanticInfo,
        ) -> Result<()> {
            *text = text.replace('e', "a");
            Ok(())
        }

        fn on_enter(
            &mut self,
            _position: &Reference,
            intro: &mut String,
            _wrap_up: &mut Option<String>,
            _semantic_info: &mut SemanticInfo,
        ) -> Result<()> {
            *intro = intro.replace('e', "a");
            Ok(())
        }

        fn on_exit(
            &mut self,
            _position: &Reference,
            _intro: &mut String,
            wrap_up: &mut Option<String>,
            _semantic_info: &mut SemanticInfo,
        ) -> Result<()> {
            *wrap_up = match wrap_up {
                Some(_) => None,
                None => Some("wot".to_owned()),
            };
            Ok(())
        }
    }

    #[test]
    fn test_mutable() {
        let mut act: Act = singleton_yaml::from_str(TEST_ACT).unwrap();
        let mut visitor = TestVisitorMut {};
        act.walk_saes_mut(&mut visitor).unwrap();
        let expected_act: Act = singleton_yaml::from_str(MODIFIED_ACT).unwrap();
        assert_eq!(act, expected_act);
    }
}
