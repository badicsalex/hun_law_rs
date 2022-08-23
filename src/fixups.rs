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

use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    identifier::ActIdentifier,
    util::{
        indentedline::{IndentedLine, IndentedLinePart},
        is_default,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fixup {
    #[serde(default, skip_serializing_if = "is_default")]
    pub after: Vec<String>,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone)]
pub struct Fixups {
    fixups: Vec<Fixup>,
    fixup_path: PathBuf,
}

impl Fixups {
    pub fn load(act_id: ActIdentifier) -> Result<Self> {
        Self::load_from(act_id, "./data/fixups/".into())
    }

    pub fn load_from(act_id: ActIdentifier, base_dir: PathBuf) -> Result<Self> {
        let fixup_path = base_dir
            .join(act_id.year.to_string())
            .join(format!("{}.yml", act_id));
        let fixups = if fixup_path.exists() {
            serde_yaml::from_reader(File::open(&fixup_path)?)?
        } else {
            Vec::new()
        };
        Ok(Self { fixups, fixup_path })
    }

    pub fn add(&mut self, f: Fixup) {
        self.fixups.push(f);
    }

    pub fn save(&self) -> Result<()> {
        create_dir_all(
            &self
                .fixup_path
                .parent()
                .ok_or_else(|| anyhow!("No parent for fixup_path"))?,
        )?;
        serde_yaml::to_writer(&mut File::create(&self.fixup_path)?, &self.fixups)?;
        Ok(())
    }

    pub fn apply(&self, lines: &mut [IndentedLine]) -> Result<()> {
        for fixup in &self.fixups {
            fixup.apply(lines)?
        }
        Ok(())
    }
}

impl From<Fixups> for Vec<Fixup> {
    fn from(f: Fixups) -> Self {
        f.fixups
    }
}

impl Fixup {
    pub fn apply(&self, lines: &mut [IndentedLine]) -> Result<()> {
        let mut needle = self.after.clone();
        needle.push(self.old.clone());
        let lines_as_strs: Vec<&str> = lines.iter().map(|l| l.content()).collect();
        let position = lines_as_strs
            .windows(needle.len())
            .position(|w| w == needle)
            .ok_or_else(|| anyhow!("Could not find '{}' in text", self.old))?
            - self.after.len();
        // BIG TODO:
        let old_line = &lines[position];
        let line = IndentedLine::from_parts(
            self.new
                .chars()
                .enumerate()
                .map(|(i, c)| IndentedLinePart {
                    dx: if i == 0 { old_line.indent() } else { 5.0 },
                    content: c,
                    bold: false,
                })
                .collect(),
            old_line.is_justified(),
        );
        lines[position] = line;
        Ok(())
    }
}
