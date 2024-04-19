use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use phazer::{Phazer, PhazerBuilder, SIMPLE_RENAME_STRATEGY, RENAME_WITH_RETRY_STRATEGY};

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
    if_error(target_path.exists(), "target_path cannot exist at this point")?;
    let p = phazer_new(&target_path);
    let rv = p.commit().map_err(|e| e.0);
    if_error(target_path.exists(), "target_path cannot exist at this point")?;
    rv
}

#[test]
fn no_writer_commit_using_default_constructor_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(
        |p| Phazer::new(p),
        "both-no-writer-commit.txt")
}

#[test]
fn no_writer_commit_using_simple_rename_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(|p| {
        PhazerBuilder::new()
            .strategy(SIMPLE_RENAME_STRATEGY)
            .path(p)
            .build()
    }, "both-no-writer-commit-simple-rename.txt")
}

#[test]
fn no_writer_commit_using_rename_with_retry_works() -> Result<(), std::io::Error> {
    no_writer_commit_works(|p| {
        PhazerBuilder::new()
            .strategy(RENAME_WITH_RETRY_STRATEGY)
            .path(p)
            .build()
    }, "both-no-writer-commit-rename-with-retry.txt")
}
