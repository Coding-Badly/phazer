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

//! Imagine, if you will, you are building an application that downloads a file from a website.
//! Let's say the application is downloading baby name data from the U.S. Social Security
//! Administration (<https://www.ssa.gov/oact/babynames/names.zip>).
//!
//! A common failure when getting data from the internet is an interrupted download.  Unless
//! precautions are taken the file ends up truncated (essentially corrupt).  That would result in a
//! bad experience for your users.  The application might stop running after outputting a cryptic
//! error regarding an unreadable ZIP file.
//!
//! A similar problem occurs with configuration files.  We want our service to only see a complete
//! configuration file.  A partial configuration file might even introduce a security
//! vulnerablility.
//!
//! The purpose of this crate is to present a file to a system in a finished state or not at all.
//! Either the entire names.zip file is downloaded or the file is missing.  Either the old complete
//! configuration file is used or the new complete configuration file is used.
//!
//! The following example shows how an interrupted application using this crate avoids putting a
//! partial file in use.
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
//!
//!     // A crash here does not corrupt the "test.cfg" file because this code was not writing
//!     // directly to "test.cfg".  If "test.cfg" did not exist before and this program crashes then
//!     // "test.cfg" still does not exist.  If "test.cfg" did exist and this program crashes then
//!     // the existing "test.cfg" is left untouched.
//!     //panic!("test.cfg never exists because of this panic.  Removing this line results in test.cfg being \"created\" atomically.");
//!
//!     writeln!(w, "Timeout=10")?;
//!     drop(w);
//!     p.commit()?;
//!     Ok(())
//! }
//! # #[cfg(not(feature = "simple"))]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//! ```
//!
//! The same code writing directly to the configuration file would leave a file behind that's
//! missing the timeout setting.  The configuration file would essentially be corrupt.
//!

mod simple_writer;
mod tokio_writer;

use std::fs::{remove_file, rename};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub trait CommitDetails {
    fn get_working_path(&self) -> &Path;
    fn get_target_path(&self) -> &Path;
    fn get_jitter(&self) -> usize;
}

pub trait CommitStrategy: Sync {
    fn commit(&self, phazer: &dyn CommitDetails) -> std::io::Result<()>;
}

/// [`Phazer`] manages the transition of a working file to a target file.
///
/// [`Phazer`] is the core component of this crate.  One [`Phazer`] is constructed for each target
/// file.  It's essentially a wrapper over the target's path.  For example, if the application
/// downloads three files from the internet then one [`Phazer`] is created for each file.
///
/// By default [`Phazer`] uses a simple rename commit strategy ([`SIMPLE_RENAME_STRATEGY`]).  When
/// [`Phazer::commit`] is called, [`rename`] is used to replace the target file with the working
/// file.  [`PhazerBuilder`] can be used to construct a [`Phazer`] with a different commit strategy.
/// The one other commit strategy availabe with this crate is [`RENAME_WITH_RETRY_STRATEGY`].
///
pub struct Phazer<'cs> {
    file_created: AtomicBool,
    commit_strategy: &'cs dyn CommitStrategy,
    working_path: PathBuf,
    target_path: PathBuf,
    phazer_id: usize,
}

impl<'cs> Phazer<'cs> {
    /// Create a Phazer where `path` is the target / destination file.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self::inner_new(path.into(), SIMPLE_RENAME_STRATEGY)
    }
    fn inner_new(target_path: PathBuf, commit_strategy: &'cs dyn CommitStrategy) -> Phazer {
        let phazer_id = current_phazer_id();
        let process_id = std::process::id();
        let lft = if let Some(ext) = target_path.extension() {
            format!("{}.phazer-", Path::new(ext).display())
        } else {
            "phazer-".into()
        };
        let rgt = format!("-{}-{}", process_id, phazer_id);
        let working_ext = format!("{}working{}", lft, rgt);
        let mut working_path = target_path.clone();
        working_path.set_extension(working_ext);
        Phazer {
            file_created: AtomicBool::new(false),
            commit_strategy,
            target_path,
            working_path,
            phazer_id,
        }
    }
    /// `commit` renames the working file so it becomes the target file.
    ///
    /// `commit` consumes the Phazer; it can only be called when there are no outstanding writers.
    pub fn commit(self) -> Result<(), std::io::Error> {
        self.commit2().map_err(|e| e.0)
    }

    pub fn commit2(self) -> Result<(), (std::io::Error, Phazer<'cs>)> {
        if self.file_created.load(Ordering::Relaxed) {
            match self.commit_strategy.commit(&self) {
                Ok(()) => Ok(()),
                Err(e) => Err((e, self)),
            }
        } else {
            Ok(())
        }
    }
    /// `first_writer` returns if the working file has not yet been created; if the caller is the
    /// one creating the first writer.  It only returns `true` once.
    #[allow(dead_code)]
    fn first_writer(&self) -> bool {
        !self.file_created.swap(true, Ordering::Relaxed)
    }
    #[cfg(feature = "test_helpers")]
    pub fn working_path(&self) -> &Path {
        &self.working_path
    }
}

impl<'cs> Drop for Phazer<'cs> {
    /// `drop` removes the working file if it still exists (if the Phazer was not committed).
    fn drop(&mut self) {
        let _ = remove_file(&self.working_path);
    }
}

impl<'cs> CommitDetails for Phazer<'cs> {
    fn get_working_path(&self) -> &Path {
        self.working_path.as_path()
    }
    fn get_target_path(&self) -> &Path {
        self.target_path.as_path()
    }
    fn get_jitter(&self) -> usize {
        self.phazer_id
    }
}

pub struct SimpleRenameStrategy {}

impl CommitStrategy for SimpleRenameStrategy {
    fn commit(&self, phazer: &dyn CommitDetails) -> std::io::Result<()> {
        rename(phazer.get_working_path(), phazer.get_target_path())
    }
}

impl Default for SimpleRenameStrategy {
    fn default() -> Self {
        Self {}
    }
}

pub const SIMPLE_RENAME_STRATEGY: &dyn CommitStrategy = &SimpleRenameStrategy {};

pub struct RenameWithRetryStrategy {}

impl CommitStrategy for RenameWithRetryStrategy {
    fn commit(&self, phazer: &dyn CommitDetails) -> std::io::Result<()> {
        let mut tries = 0;
        let jitter = (phazer.get_jitter() as u64) & 0xF;
        let base_sleep = 11 + (3 * jitter);
        loop {
            tries += 1;
            let rv = rename(phazer.get_working_path(), phazer.get_target_path());
            match &rv {
                Ok(()) => return rv,
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::PermissionDenied {
                        return rv;
                    }
                    // With 10 threads and the sleep code as it is below (start with 10ms), seven
                    // has been a good threshold.
                    if tries >= 7 {
                        return rv;
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(base_sleep * tries));
        }
    }
}

impl Default for RenameWithRetryStrategy {
    fn default() -> Self {
        Self {}
    }
}

pub const RENAME_WITH_RETRY_STRATEGY: &dyn CommitStrategy = &RenameWithRetryStrategy {};

pub struct PhazerBuilder<'cs> {
    commit_strategy: Option<&'cs dyn CommitStrategy>,
}

pub struct PhazerBuilderWithPath<'cs> {
    commit_strategy: Option<&'cs dyn CommitStrategy>,
    target_path: PathBuf,
}

impl<'cs> PhazerBuilder<'cs> {
    pub fn new() -> Self {
        Self {
            commit_strategy: None,
        }
    }
    pub fn with_path<P>(path: P) -> PhazerBuilderWithPath<'cs>
    where
        P: Into<PathBuf>,
    {
        PhazerBuilderWithPath {
            commit_strategy: None,
            target_path: path.into(),
        }
    }
    pub fn path<P>(self, value: P) -> PhazerBuilderWithPath<'cs>
    where
        P: Into<PathBuf>,
    {
        PhazerBuilderWithPath {
            commit_strategy: self.commit_strategy,
            target_path: value.into(),
        }
    }
    pub fn strategy(mut self, value: &'cs dyn CommitStrategy) -> Self {
        self.commit_strategy = Some(value);
        self
    }
}

impl<'cs> PhazerBuilderWithPath<'cs> {
    pub fn path<P>(mut self, value: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.target_path = value.into();
        self
    }
    pub fn strategy(mut self, value: &'cs dyn CommitStrategy) -> Self {
        self.commit_strategy = Some(value);
        self
    }
    pub fn build(self) -> Phazer<'cs> {
        let Self {
            commit_strategy,
            target_path,
        } = self;
        let commit_strategy = commit_strategy.unwrap_or(SIMPLE_RENAME_STRATEGY);
        Phazer::inner_new(target_path, commit_strategy)
    }
}

// Return a serial number for this application to ensure the working filename is unique.
fn current_phazer_id() -> usize {
    static NEXT_PHAZER_ID: AtomicUsize = AtomicUsize::new(0);
    NEXT_PHAZER_ID.fetch_add(1, Ordering::Relaxed)
}
