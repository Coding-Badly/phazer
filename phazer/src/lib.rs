// Copyright 2023 Brian Cook (a.k.a. Coding-Badly)
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

//! Imagine, if you will, that you are building an application that downloads a file from a website.
//! Let's say that the application is downloading the baby name data from the U.S. Social Security
//! Administration (https://www.ssa.gov/oact/babynames/names.zip).
//!
//! A common failure when getting data from the internet is an interrupted download.  Unless
//! precautions are taken the file ends up truncated (essentially corrupt).  That would result in a
//! bad experience your users.  The application might stop running after outputting a cryptic error
//! regarding an unreadable ZIP file.
//!
//! A similar problem occurs with configuration files.  We want our service to only see a complete
//! configuration file.  A partial configuration file might even introduce a security
//! vulnerablility.
//!
//! The purpose of this crate is to present a file to a system in a finished state or not at all.
//! Either the entire names.zip file is downloaded or the file is missing.  Either the old complete
//! configuration file is used or the new complete configuration file is used.
//!

//! The following example shows how an interrupted application avoids putting a partial file in use.
//!
//! ```rust
//! # use std::io::Write;
//! #
//! # use phazer::Phazer;
//! #
//! # #[cfg(feature = "simple")]
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let p = Phazer::new("test.cfg");
//!     let mut w = p.simple_writer()?;
//!     writeln!(w, "[Settings]")?;
//!     writeln!(w, "Port=1")?;
//!     panic!("test.cfg never exists because of this panic.  Removing this line results in test.cfg being \"created\" atomically.");
//!     writeln!(w, "Timeout=10")?;
//!     p.commit()?;
//!     Ok(())
//! }
//! # #[cfg(not(feature = "simple"))]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//! ```

mod simple_writer;
mod tokio_writer;

#[cfg(any(feature = "simple", feature = "tokio"))]
use std::cell::Cell;
use std::fs::{remove_file, rename};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Phazer is the entry point into this crate.
///
/// One Phazer is constructed for each file that's created.  It's essentially a wrapper over the
/// target filename.  For example, if the application downloads three files from the internet then
/// one Phazer is created for each file.
pub struct Phazer {
    #[cfg(any(feature = "simple", feature = "tokio"))]
    file_created: Cell<bool>,
    working_path: PathBuf,
    target_path: PathBuf,
}

impl Phazer {
    /// Create a Phazer where `path` is the target / destination file.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let target_path = path.into();
        let phazer_id = current_phazer_id();
        let process_id = std::process::id();
        let working_ext = if let Some(ext) = target_path.extension() {
            format!(
                "{}.phazer-{}-{}",
                Path::new(ext).display(),
                process_id,
                phazer_id
            )
        } else {
            format!("phazer-{}-{}", process_id, phazer_id)
        };
        let mut working_path = target_path.clone();
        working_path.set_extension(working_ext);
        Self {
            #[cfg(any(feature = "simple", feature = "tokio"))]
            file_created: Cell::new(false),
            target_path,
            working_path,
        }
    }
    /// `commit` renames the working file so it becomes the target file.
    ///
    /// `commit` consumes the Phazer; it can only be called when there are no outstanding writers.
    pub fn commit(self) -> std::io::Result<()> {
        rename(&self.working_path, &self.target_path)?;
        Ok(())
    }
    ///
    ///
    #[cfg(feature = "bug-001")]
    pub fn working_path(&self) -> &Path {
        &self.working_path.as_path()
    }
}

impl Drop for Phazer {
    /// `drop` removes the working file if it still exists (if the Phazer was not committed).
    fn drop(&mut self) {
        let _ = remove_file(&self.working_path);
    }
}

// Return a serial number for this application to ensure the working filename is unique.
fn current_phazer_id() -> usize {
    static NEXT_PHAZER_ID: AtomicUsize = AtomicUsize::new(0);
    NEXT_PHAZER_ID.fetch_add(1, Ordering::Relaxed)
}
