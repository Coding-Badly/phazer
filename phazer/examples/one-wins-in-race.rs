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

#[cfg(feature = "simple")]
mod inner {
    use std::borrow::Cow;
    use std::fs::{canonicalize, create_dir, read_to_string, remove_file};
    use std::io::{ErrorKind, Write};
    use std::path::PathBuf;
    use std::thread::{scope, ScopedJoinHandle};

    use phazer::{CommitStrategy, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY};

    trait IgnoreThese {
        fn ignore(&self, kind: ErrorKind) -> bool;
    }

    fn ignore<T, I>(r: Result<T, std::io::Error>, these: I) -> Result<Option<T>, std::io::Error>
    where
        I: IgnoreThese,
    {
        match r {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                if these.ignore(e.kind()) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    impl IgnoreThese for ErrorKind {
        fn ignore(&self, kind: ErrorKind) -> bool {
            *self == kind
        }
    }

    fn prepare_working_dir() -> Result<PathBuf, std::io::Error> {
        let mut working_dir = canonicalize(".")?;
        working_dir.push("local");
        ignore(create_dir(&working_dir), ErrorKind::AlreadyExists)?;
        Ok(working_dir)
    }

    struct DoOneResult {
        error_count: usize,
        worked: bool,
    }

    impl std::fmt::Display for DoOneResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let text = match (self.error_count == 0, self.worked) {
                (true, true) => Cow::Borrowed("No errors and at least one thread won.  Success!"),
                (false, true) => Cow::Owned(format!("{} errors and at least one thread won.  A partial success.", self.error_count)),
                (_, false) => Cow::Borrowed("Yikes!  No threads won.  That's a failure."),
            };
            f.pad(&text)
        }
    }

    fn do_one(commit_strategy: &(dyn CommitStrategy + Sync)) -> Result<DoOneResult, Box<dyn std::error::Error>> {
        let working_dir = prepare_working_dir()?;
        let target_path = working_dir.join("one-wins-in-race.txt");
        ignore(remove_file(&target_path), ErrorKind::NotFound)?;

        let contents = vec!["first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth"];

        println!("{} threads racing to win the target file {}....", contents.len(), target_path.display());
        let results: Vec<_> = scope(|s| {
            let mut join_handles = Vec::<ScopedJoinHandle<'_, Result<(), std::io::Error>>>::new();
            for content in contents.iter() {
                let tpc = target_path.clone();
                join_handles.push(
                    s.spawn(move || {
                        let p = PhazerBuilder::with_path(tpc)
                            .strategy(commit_strategy)
                            .build();
                        let mut w = p.simple_writer()?;
                        w.write_all(content.as_bytes())?;
                        drop(w);
                        p.commit().map_err(|v| v.0)?;
                        Ok(())
                    })
                );
            }
            join_handles
                .into_iter()
                .map(|h| h.join().expect("a thread failed to start"))
                .collect()
        });
        let errors: Vec<_> = results
            .into_iter()
            .filter_map(|r| {
                match r {
                    Ok(()) => None,
                    Err(e) => Some(e),
                }
            })
            .collect();
        let error_count = errors.len();
        if errors.len() > 0 {
            println!("The following errors occurred...");
            for e in errors.into_iter() {
                println!("{}", e);
            }
        } else {
            println!("No errors.");
        }
        let worked;
        let s = read_to_string(&target_path)?;
        if let Some(f) = contents.iter().find(|v| **v == s) {
            println!("{} won.", f);
            worked = true;
        } else {
            println!("No winner found.");
            worked = false;
        }
        Ok(DoOneResult {
            error_count,
            worked,
        })
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();

        println!("Let's start with a simple rename strategy...");
        let dor = do_one(SIMPLE_RENAME_STRATEGY)?;
        println!("{}", dor);
        println!();

        println!("Let's run the same test but retry when an error occurs...");
        let dor = do_one(RENAME_WITH_RETRY_STRATEGY)?;
        println!("{}", dor);
        println!();

        println!();
        Ok(())
    }
}

#[cfg(not(feature = "simple"))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!(
            "This example requires the 'simple' feature to be enabled.  Try..."
        );
        println!("cargo run --example one-wins-in-race --features simple");
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
