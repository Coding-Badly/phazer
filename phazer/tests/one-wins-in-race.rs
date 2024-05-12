mod common;

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

    struct DoOneResult {
        errors: Vec<std::io::Error>,
        winner: Option<String>,
    }

    fn do_one<P>(
        filename: P,
        commit_strategy: &(dyn CommitStrategy + Sync),
    ) -> Result<DoOneResult, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;
        let contents = vec![
            "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
            "tenth",
        ];
        // contents.len() threads racing to win the target file target_path.display()
        let results: Vec<_> = scope(|s| {
            let mut join_handles = Vec::<ScopedJoinHandle<'_, Result<(), std::io::Error>>>::new();
            for content in contents.iter() {
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
        let winner = if let Some(f) = contents.iter().find(|v| **v == s) {
            Some(f.to_string())
        } else {
            None
        };
        let _ = remove_file(&target_path);

        Ok(DoOneResult { errors, winner })
    }

    #[test]
    fn using_simple_rename() -> Result<(), std::io::Error> {
        let dor = do_one("one-wins-in-race-simple-rename.txt", SIMPLE_RENAME_STRATEGY)?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // If there are errors (there always has been) they are all permission denied
        for error in dor.errors.iter() {
            assert!(error.kind() == std::io::ErrorKind::PermissionDenied);
        }
        Ok(())
    }

    #[test]
    fn using_rename_with_retry() -> Result<(), std::io::Error> {
        let dor = do_one(
            "one-wins-in-race-rename-with-retry.txt",
            RENAME_WITH_RETRY_STRATEGY,
        )?;
        // Under Windows the strategy works
        assert!(dor.winner.is_some());
        // There should be no errors
        assert!(dor.errors.len() == 0);
        Ok(())
    }
}
