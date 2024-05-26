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

    use crate::common::{
        prepare_target_file, WRITE_COMMIT_SIMPLE_DEFAULT, WRITE_COMMIT_SIMPLE_RENAME,
        WRITE_COMMIT_SIMPLE_WITH_RETRY,
    };

    use super::if_error;

    fn write_commit_works<C, P>(
        phazer_new: C,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(
            target_path.exists(),
            "target_path cannot exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.simple_writer()?;
        w.write_all("first".as_bytes())?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(
            !working_path.exists(),
            "working_path must exist at this point",
        )?;
        if_error(
            target_path.exists(),
            "target_path cannot exist at this point",
        )?;

        // Commit the first version
        p.commit()?;

        // At this point the target should exist and the working file should be gone
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Ensure the target has the expected data
        let s = read_to_string(&target_path)?;
        if_error(s != "first", "target_path file must contain \"first\" ")?;

        // Do it all again
        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point the target must exist and the working file must not
        if_error(
            !target_path.exists(),
            "target_path cannot exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.simple_writer()?;
        w.write_all("second".as_bytes())?;
        drop(w);

        // At this point both files should exist
        if_error(
            !working_path.exists(),
            "working_path must exist at this point",
        )?;
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;

        // Commit the first version
        p.commit()?;

        // At this point the target should exist and the working file should be gone
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Ensure the target has the expected data
        let s = read_to_string(&target_path)?;
        if_error(s != "second", "target_path file must contain \"second\" ")?;

        let _ = remove_file(&target_path);

        Ok(())
    }

    #[test]
    fn write_commit_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>> {
        write_commit_works(|p| Phazer::new(p), WRITE_COMMIT_SIMPLE_DEFAULT)
    }

    #[test]
    fn write_commit_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_commit_works(
            |p| {
                PhazerBuilder::new()
                    .strategy(SIMPLE_RENAME_STRATEGY)
                    .path(p)
                    .build()
            },
            WRITE_COMMIT_SIMPLE_RENAME,
        )
    }

    #[test]
    fn write_commit_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>> {
        write_commit_works(
            |p| {
                PhazerBuilder::new()
                    .strategy(RENAME_WITH_RETRY_STRATEGY)
                    .path(p)
                    .build()
            },
            WRITE_COMMIT_SIMPLE_WITH_RETRY,
        )
    }
}

#[cfg(all(feature = "tokio", feature = "test_helpers"))]
mod tokio {
    use std::path::{Path, PathBuf};

    use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY, SIMPLE_RENAME_STRATEGY};
    use tokio::fs::{read_to_string, remove_file};
    use tokio::io::AsyncWriteExt;

    use crate::common::{
        prepare_target_file, WRITE_COMMIT_TOKIO_DEFAULT, WRITE_COMMIT_TOKIO_RENAME,
        WRITE_COMMIT_TOKIO_WITH_RETRY,
    };

    use super::if_error;

    async fn write_commit_works<C, P>(
        phazer_new: C,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point neither file should exist
        if_error(
            target_path.exists(),
            "target_path cannot exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.tokio_writer().await?;
        w.write_all("first".as_bytes()).await?;
        drop(w);

        // At this point the working file should exist but not the target
        if_error(
            !working_path.exists(),
            "working_path must exist at this point",
        )?;
        if_error(
            target_path.exists(),
            "target_path cannot exist at this point",
        )?;

        // Commit the first version
        p.commit()?;

        // At this point the target should exist and the working file should be gone
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Ensure the target has the expected data
        let s = read_to_string(&target_path).await?;
        if_error(s != "first", "target_path file must contain \"first\" ")?;

        // Do it all again
        let p = phazer_new(&target_path);
        let working_path = p.working_path().to_path_buf();

        // At this point the target must exist and the working file must not
        if_error(
            !target_path.exists(),
            "target_path cannot exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Create the working file, write some stuff, then close the working file
        let mut w = p.tokio_writer().await?;
        w.write_all("second".as_bytes()).await?;
        drop(w);

        // At this point both files should exist
        if_error(
            !working_path.exists(),
            "working_path must exist at this point",
        )?;
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;

        // Commit the first version
        p.commit()?;

        // At this point the target should exist and the working file should be gone
        if_error(
            !target_path.exists(),
            "target_path must exist at this point",
        )?;
        if_error(
            working_path.exists(),
            "working_path cannot exist at this point",
        )?;

        // Ensure the target has the expected data
        let s = read_to_string(&target_path).await?;
        if_error(s != "second", "target_path file must contain \"second\" ")?;

        let _ = remove_file(&target_path).await;

        Ok(())
    }

    #[tokio::test]
    async fn write_commit_using_default_constructor_works() -> Result<(), Box<dyn std::error::Error>>
    {
        write_commit_works(|p| Phazer::new(p), WRITE_COMMIT_TOKIO_DEFAULT).await
    }

    #[tokio::test]
    async fn write_commit_using_simple_rename_works() -> Result<(), Box<dyn std::error::Error>> {
        write_commit_works(
            |p| {
                PhazerBuilder::new()
                    .strategy(SIMPLE_RENAME_STRATEGY)
                    .path(p)
                    .build()
            },
            WRITE_COMMIT_TOKIO_RENAME,
        )
        .await
    }

    #[tokio::test]
    async fn write_commit_using_rename_with_retry_works() -> Result<(), Box<dyn std::error::Error>>
    {
        write_commit_works(
            |p| {
                PhazerBuilder::new()
                    .strategy(RENAME_WITH_RETRY_STRATEGY)
                    .path(p)
                    .build()
            },
            WRITE_COMMIT_TOKIO_WITH_RETRY,
        )
        .await
    }
}
