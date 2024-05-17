// Copyright 2024 Brian Cook (a.k.a. Coding-Badly)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY};

mod common;

use crate::common::prepare_target_file;

fn if_error(value: bool, text: &'static str) -> Result<(), std::io::Error> {
    if value {
        Err(Error::new(ErrorKind::Other, text))
    } else {
        Ok(())
    }
}

fn no_writer_commit_works<C, P>(phazer_new: C, filename: P) -> Result<(), std::io::Error>
where
    C: Fn(&PathBuf) -> Phazer,
    P: AsRef<Path>,
{
    let target_path = prepare_target_file(filename)?;
    if_error(
        target_path.exists(),
        "target_path cannot exist at this point",
    )?;
    let p = phazer_new(&target_path);
    let rv = p.commit();
    if_error(
        target_path.exists(),
        "target_path cannot exist at this point",
    )?;
    rv
}

#[test]
fn no_writer_commit_using_default_constructor_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(|p| Phazer::new(p), "both-no-writer-commit.txt")
}

#[test]
fn no_writer_commit_using_simple_rename_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(
        |p| {
            PhazerBuilder::new()
                .strategy(SIMPLE_RENAME_STRATEGY)
                .path(p)
                .build()
        },
        "both-no-writer-commit-simple-rename.txt",
    )
}

#[test]
fn no_writer_commit_using_rename_with_retry_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(
        |p| {
            PhazerBuilder::new()
                .strategy(RENAME_WITH_RETRY_STRATEGY)
                .path(p)
                .build()
        },
        "both-no-writer-commit-rename-with-retry.txt",
    )
}
