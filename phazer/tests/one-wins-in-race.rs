mod common;

#[allow(dead_code)]
struct DoOneResult {
    errors: Vec<std::io::Error>,
    winner: Option<String>,
}

#[allow(dead_code)]
const CONTENTS: [&'static str; 10] = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
];

#[cfg(feature = "simple")]
mod simple {
    use std::fs::{read_to_string, remove_file};
    use std::io::Write;
    use std::path::Path;
    use std::thread::{scope, ScopedJoinHandle};

    use phazer::{
        CommitStrategy, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY,
    };

    use crate::common::prepare_target_file;

    use super::{DoOneResult, CONTENTS};

    fn do_one<P>(
        filename: P,
        commit_strategy: &dyn CommitStrategy,
    ) -> Result<DoOneResult, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;
        // CONTENTS.len() threads racing to win the target file target_path.display()
        let results: Vec<_> = scope(|s| {
            let mut join_handles = Vec::<ScopedJoinHandle<'_, Result<(), std::io::Error>>>::new();
            for content in CONTENTS.iter() {
                let tpc = target_path.clone();
                join_handles.push(s.spawn(move || {
                    let p = PhazerBuilder::with_path(tpc)
                        .strategy(commit_strategy)
                        .build();
                    let mut w = p.simple_writer()?;
                    w.write_all(content.as_bytes())?;
                    drop(w);
                    p.commit()?;
                    Ok(())
                }));
            }
            join_handles
                .into_iter()
                .map(|h| h.join().expect("a thread failed to start"))
                .collect()
        });
        let errors: Vec<_> = results
            .into_iter()
            .filter_map(|r| match r {
                Ok(()) => None,
                Err(e) => Some(e),
            })
            .collect();
        let s = read_to_string(&target_path)?;
        let winner = if let Some(f) = CONTENTS.iter().find(|v| **v == s) {
            Some(f.to_string())
        } else {
            None
        };
        let _ = remove_file(&target_path);

        Ok(DoOneResult { errors, winner })
    }

    #[test]
    fn using_simple_rename() -> Result<(), std::io::Error> {
        let dor = do_one(
            "simple-one-wins-in-race-simple-rename.txt",
            SIMPLE_RENAME_STRATEGY,
        )?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // POSIX.1-2001 requires the rename to be atomic which implies that, if a single thread is
        // able to rename, all threads will be able to rename.  In which case we expect zero errors.
        #[cfg(unix)]
        {
            // There should be no errors
            assert!(dor.errors.len() == 0);
        }
        // If there are errors (there always has been with Windows) ensure they are all permission
        // denied
        for error in dor.errors.iter() {
            assert!(error.kind() == std::io::ErrorKind::PermissionDenied);
        }
        Ok(())
    }

    #[test]
    fn using_rename_with_retry() -> Result<(), std::io::Error> {
        let dor = do_one(
            "simple-one-wins-in-race-rename-with-retry.txt",
            RENAME_WITH_RETRY_STRATEGY,
        )?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // There should be no errors
        assert!(dor.errors.len() == 0);
        Ok(())
    }
}

#[cfg(feature = "tokio")]
mod tokio {
    use std::path::Path;

    use futures::future::join_all;
    use phazer::{
        CommitStrategy, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY,
    };
    use tokio::fs::{read_to_string, remove_file};
    use tokio::io::AsyncWriteExt;
    use tokio::task::JoinHandle;

    use crate::common::prepare_target_file;

    use super::{DoOneResult, CONTENTS};

    async fn do_one<P>(
        filename: P,
        commit_strategy: &'static dyn CommitStrategy,
    ) -> Result<DoOneResult, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;
        // CONTENTS.len() threads racing to win the target file target_path.display()
        let mut join_handles = Vec::<JoinHandle<Result<(), std::io::Error>>>::new();
        for content in CONTENTS.iter() {
            let tpc = target_path.clone();
            join_handles.push(tokio::spawn(async {
                let p = PhazerBuilder::with_path(tpc)
                    .strategy(commit_strategy)
                    .build();
                let mut w = p.tokio_writer().await?;
                w.write_all(content.as_bytes()).await?;
                drop(w);
                p.commit()?;
                Ok(())
            }));
        }
        let results: Vec<_> = join_all(join_handles).await;
        let errors: Vec<_> = results
            .into_iter()
            .map(|h| h.expect("a task failed to start"))
            .filter_map(|r| match r {
                Ok(()) => None,
                Err(e) => Some(e),
            })
            .collect();
        let s = read_to_string(&target_path).await?;
        let winner = if let Some(f) = CONTENTS.iter().find(|v| **v == s) {
            Some(f.to_string())
        } else {
            None
        };
        let _ = remove_file(&target_path).await;

        Ok(DoOneResult { errors, winner })
    }

    #[tokio::test]
    async fn using_simple_rename() -> Result<(), std::io::Error> {
        let dor = do_one(
            "tokio-one-wins-in-race-simple-rename.txt",
            SIMPLE_RENAME_STRATEGY,
        )
        .await?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // POSIX.1-2001 requires the rename to be atomic which implies that, if a single thread is
        // able to rename, all threads will be able to rename.  In which case we expect zero errors.
        #[cfg(unix)]
        {
            // There should be no errors
            assert!(dor.errors.len() == 0);
        }
        // If there are errors (there always has been with Windows) ensure they are all permission
        // denied
        for error in dor.errors.iter() {
            assert!(error.kind() == std::io::ErrorKind::PermissionDenied);
        }
        Ok(())
    }

    #[tokio::test]
    async fn using_rename_with_retry() -> Result<(), std::io::Error> {
        let dor = do_one(
            "tokio-one-wins-in-race-rename-with-retry.txt",
            RENAME_WITH_RETRY_STRATEGY,
        )
        .await?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // There should be no errors
        assert!(dor.errors.len() == 0);
        Ok(())
    }
}
