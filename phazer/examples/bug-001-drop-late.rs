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

#[cfg(all(feature = "bug-001", feature = "simple"))]
mod inner {
    use std::fs::{canonicalize, create_dir, File, OpenOptions};
    use std::io::{ErrorKind, Write};
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::{Path, PathBuf};

    use phazer::Phazer;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_DELETE_ON_CLOSE;

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

    fn make_file_delete_on_close<P: AsRef<Path>>(path: P) -> Result<File, std::io::Error> {
        OpenOptions::new()
            .write(true)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(path)
    }

    enum LocalError {
        CommitWorked,
        IoError(std::io::Error),
    }

    impl std::fmt::Debug for LocalError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Self as std::fmt::Display>::fmt(self, f)
        }
    }

    impl std::fmt::Display for LocalError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::CommitWorked => f.pad("Phazer::commit worked but was expected to fail"),
                Self::IoError(e) => {
                    let text = format!("Phazer::commit failed; the error message is {}", e);
                    f.pad(&text)
                }
            }
        }
    }

    impl std::error::Error for LocalError {}

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let working_dir = prepare_working_dir()?;
        let target_path = working_dir.join("bug-001-drop-late.txt");
        let phazer = Phazer::new(&target_path);

        // Create the writer we'll be using
        let mut writer = phazer.simple_writer()?;

        // Make the working file delete-on-close
        let _docf = make_file_delete_on_close(phazer.working_path())?;
        drop(_docf);

        // Write some stuff
        write!(writer, "This is a test.  This is only a test.  Had this not been a test something interesting would be here.")?;
        // The writer now must be explicitly dropped with the addition of SimplePhazerWriter::Drop.
        // With the working file closed before the commit this code works as expected (file not
        // found error).
        drop(writer);

        // Because the working file is delete-on-close, if the working file really was closed at
        // this point, the commit would fail with...
        //   Error: Os { code: 2, kind: NotFound, message: "The system cannot find the file specified." }
        //
        // If the `drop(docf)` is removed then the commit works but the file disappears at the end
        // of this function when docf and the writer are actually dropped.
        //
        // That's also the expected behaviour when the `drop(docf)` is included.  Instead the commit
        // fails with...
        //   Error: Os { code: 5, kind: PermissionDenied, message: "Access is denied." }

        // Try to commit.
        match phazer.commit() {
            Ok(()) => {
                // This is a failure.  The working file should not exist.
                Err(LocalError::CommitWorked.into())
            }
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    // This is a failure.  The working file should not exist.
                    Err(LocalError::IoError(e).into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(not(all(feature = "bug-001", feature = "simple")))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!(
            "This example requires the 'bug-001' and 'simple' features to be enabled.  Try..."
        );
        println!("cargo run --example bug-001-drop-late --features bug-001,simple");
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
