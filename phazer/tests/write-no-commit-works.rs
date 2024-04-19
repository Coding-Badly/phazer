mod common;

use std::io::{Error, ErrorKind};

#[allow(unused)]
fn if_error(value: bool, text: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    if value {
        Err(Error::new(ErrorKind::Other, text).into())
    } else {
        Ok(())
    }
}

#[cfg(all(feature = "simple", feature = "test_helpers"))]
mod simple {
    use std::fs::{read_to_string, remove_file};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY};

    use crate::common::prepare_target_file;

    use super::if_error;

    fn write_no_commit_no_target_works<C, P>(phazer_new: C, filename: P) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.simple_writer()?;
        w.write_all("cannot become the target".as_bytes())?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(target_path.exists(), "target_path cannot exist at this point")?;

        drop(p);

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        Ok(())
    }

    fn write_no_commit_have_target_works<C, P>(phazer_new: C, filename: P) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write nothing, then close the working file
        let w = p.simple_writer()?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(target_path.exists(), "target_path cannot exist at this point")?;

        // Commit the empty file
        p.commit().map_err(|v| v.0)?;

        // At this point the target should exist and the working file should be gone
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Ensure the target is empty
        let s = read_to_string(&target_path)?;
        if_error(s.len() > 0, "target_path file must be empty; nothing was written")?;

        // Do it again.  This time without the commit.

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point the target should exist and the working file should not
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write nothing, then close the working file
        let mut w = p.simple_writer()?;
        w.write_all("cannot become the target".as_bytes())?;
        drop(w);

        // At this point both files should exist
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(!target_path.exists(), "target_path must exist at this point")?;

        drop(p);

        // At this point the target should exist and the working file should not
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Ensure the target is empty
        let s = read_to_string(&target_path)?;
        if_error(s.len() > 0, "target_path file must be empty; no commit")?;

        let _ = remove_file(&target_path);

        Ok(())
    }

    #[test]
    fn write_no_commit_no_target_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(
            |p| Phazer::new(p),
            "simple-write-no-commit-no-target-works.txt")
    }

    #[test]
    fn write_no_commit_no_target_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(|p| {
            PhazerBuilder::new()
                .strategy(SIMPLE_RENAME_STRATEGY)
                .path(p)
                .build()
        }, "simple-write-no-commit-no-target-simple-rename-works.txt")
    }

    #[test]
    fn write_no_commit_no_target_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(|p| {
            PhazerBuilder::new()
                .strategy(RENAME_WITH_RETRY_STRATEGY)
                .path(p)
                .build()
        }, "simple-write-no-commit-no-target-rename-with-retry-works.txt")
    }

    #[test]
    fn write_no_commit_have_target_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(
            |p| Phazer::new(p),
            "simple-write-no-commit-have-target-works.txt")
    }

    #[test]
    fn write_no_commit_have_target_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(|p| {
            PhazerBuilder::new()
                .strategy(SIMPLE_RENAME_STRATEGY)
                .path(p)
                .build()
        }, "simple-write-no-commit-have-target-simple-rename-works.txt")
    }

    #[test]
    fn write_no_commit_have_target_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(|p| {
            PhazerBuilder::new()
                .strategy(RENAME_WITH_RETRY_STRATEGY)
                .path(p)
                .build()
        }, "simple-write-no-commit-have-target-rename-with-retry-works.txt")
    }

}

#[cfg(all(feature = "tokio", feature = "test_helpers"))]
mod tokio {
    use std::fs::{read_to_string, remove_file};
    use std::path::{Path, PathBuf};

    use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY};
    use tokio::io::AsyncWriteExt;

    use crate::common::prepare_target_file;

    use super::if_error;

    async fn write_no_commit_no_target_works<C, P>(phazer_new: C, filename: P) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.tokio_writer().await?;
        w.write_all("cannot become the target".as_bytes()).await?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(target_path.exists(), "target_path cannot exist at this point")?;

        drop(p);

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        Ok(())
    }

    async fn write_no_commit_have_target_works<C, P>(phazer_new: C, filename: P) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(target_path.exists(), "target_path cannot exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write nothing, then close the working file
        let w = p.tokio_writer().await?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(target_path.exists(), "target_path cannot exist at this point")?;

        // Commit the empty file
        p.commit().map_err(|v| v.0)?;

        // At this point the target should exist and the working file should be gone
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Ensure the target is empty
        let s = read_to_string(&target_path)?;
        if_error(s.len() > 0, "target_path file must be empty; nothing was written")?;

        // Do it again.  This time without the commit.

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point the target should exist and the working file should not
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Create the working file, write nothing, then close the working file
        let mut w = p.tokio_writer().await?;
        w.write_all("cannot become the target".as_bytes()).await?;
        drop(w);

        // At this point both files should exist
        if_error(!working_path.exists(), "working_path must exist at this point")?;
        if_error(!target_path.exists(), "target_path must exist at this point")?;

        drop(p);

        // At this point the target should exist and the working file should not
        if_error(!target_path.exists(), "target_path must exist at this point")?;
        if_error(working_path.exists(), "working_path cannot exist at this point")?;

        // Ensure the target is empty
        let s = read_to_string(&target_path)?;
        if_error(s.len() > 0, "target_path file must be empty; no commit")?;

        let _ = remove_file(&target_path);

        Ok(())
    }

    #[tokio::test]
    async fn write_no_commit_no_target_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(
            |p| Phazer::new(p),
            "tokio-write-no-commit-no-target-works.txt").await
    }

    #[tokio::test]
    async fn write_no_commit_no_target_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(|p| {
            PhazerBuilder::new()
                .strategy(SIMPLE_RENAME_STRATEGY)
                .path(p)
                .build()
        }, "tokio-write-no-commit-no-target-simple-rename-works.txt").await
    }

    #[tokio::test]
    async fn write_no_commit_no_target_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_no_target_works(|p| {
            PhazerBuilder::new()
                .strategy(RENAME_WITH_RETRY_STRATEGY)
                .path(p)
                .build()
        }, "tokio-write-no-commit-no-target-rename-with-retry-works.txt").await
    }

    #[tokio::test]
    async fn write_no_commit_have_target_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(
            |p| Phazer::new(p),
            "tokio-write-no-commit-have-target-works.txt").await
    }

    #[tokio::test]
    async fn write_no_commit_have_target_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(|p| {
            PhazerBuilder::new()
                .strategy(SIMPLE_RENAME_STRATEGY)
                .path(p)
                .build()
        }, "tokio-write-no-commit-have-target-simple-rename-works.txt").await
    }

    #[tokio::test]
    async fn write_no_commit_have_target_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>> {
        write_no_commit_have_target_works(|p| {
            PhazerBuilder::new()
                .strategy(RENAME_WITH_RETRY_STRATEGY)
                .path(p)
                .build()
        }, "tokio-write-no-commit-have-target-rename-with-retry-works.txt").await
    }

}
