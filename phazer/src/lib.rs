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
// rmv use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub enum RetryStrategyAction {
    ReturnError,
    TryAgain,
}

pub trait RetryStrategy {
    fn before_commit(&mut self);
    fn handle_error(&mut self, tries: i32) -> RetryStrategyAction;
}

pub struct Never {}

impl Default for Never {
    fn default() -> Self {
        Self {}
    }
}

impl RetryStrategy for Never {
    fn before_commit(&mut self) {}
    fn handle_error(&mut self, _tries: i32) -> RetryStrategyAction {
        RetryStrategyAction::ReturnError
    }
}

pub struct ThreeTries {}

impl ThreeTries {
    #[cfg(not(feature = "tokio"))]
    fn short_nap(tries: i32) {
        std::thread::sleep(std::time::Duration::from_millis(tries as u64 * 125));
    }
}

impl Default for ThreeTries {
    fn default() -> Self {
        Self {}
    }
}

impl RetryStrategy for ThreeTries {
    fn before_commit(&mut self) {}
    fn handle_error(&mut self, tries: i32) -> RetryStrategyAction {
        if tries > 3 {
            return RetryStrategyAction::ReturnError;
        }
        Self::short_nap(tries);
        RetryStrategyAction::TryAgain
    }
}

// rmv pub enum RetryStrategyRmv  {
// rmv     Never,
// rmv     Repeat,
// rmv     ForgeAhead,
// rmv }

/// Phazer is the entry point into this crate.
///
/// One Phazer is constructed for each file that's created.  It's essentially a wrapper over the
/// target filename.  For example, if the application downloads three files from the internet then
/// one Phazer is created for each file.
pub struct Phazer {
    #[cfg(any(feature = "simple", feature = "tokio"))]
    file_created: Cell<bool>,
    commit_tried: bool,
    last_commit_error: Option<std::io::Error>,
    retry_strategy: Box<dyn RetryStrategy>,
    // rmv retry_strategy_rmv: RetryStrategyRmv,
    target_path: PathBuf,
    working_path: PathBuf,
}

impl Phazer {
    /// Create a Phazer where `path` is the target / destination file.
    pub fn new<P>(path: P, retry_strategy: impl RetryStrategy + 'static) -> Self
    where
        P: Into<PathBuf>,
    {
        Self::init(path.into(), Box::new(retry_strategy))
    }
    /// `commit` renames the working file so it becomes the target file.
    ///
    /// `commit` consumes the Phazer; it can only be called when there are no outstanding writers.
    pub fn commit(mut self) -> Result<(), Self> {
        self.commit_tried = true;
        self.last_commit_error = None;
        let mut tries = 0;
        self.retry_strategy.before_commit();
        loop {
            tries += 1;
            match self.internal_commit() {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if self.last_commit_error.is_none() {
                        self.last_commit_error = Some(e);
                    }
                }
            }
            match self.retry_strategy.handle_error(tries) {
                RetryStrategyAction::ReturnError => return Err(self),
                RetryStrategyAction::TryAgain => {}
            }
            // rmv match self.retry_strategy_rmv {
            // rmv     RetryStrategyRmv::Never => return Err(self),
            // rmv     RetryStrategyRmv::Repeat => {
            // rmv         if tries >= 3 {
            // rmv             return Err(self);
            // rmv         }
            // rmv         Self::short_nap(tries);
            // rmv     }
            // rmv     RetryStrategyRmv::ForgeAhead => {
            // rmv         // nfx: Try a two-phase commit.
            // rmv     }
            // rmv }
        }
        /* rmv
        Ok(())
        match rename(&self.working_path, &self.target_path) {
            Ok(()) => {}
            Err(e) => {
                if e.kind() != ErrorKind::PermissionDenied {
                    return Err(e);
                }
                let _ = remove_file(&self.target_path);
                match rename(&self.working_path, &self.target_path) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        */
    }
    fn init(target_path: PathBuf, retry_strategy: Box<dyn RetryStrategy>) -> Self {
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
            commit_tried: false,
            last_commit_error: None,
            retry_strategy,
            // rmv retry_strategy_rmv: RetryStrategyRmv::Never,
            target_path,
            working_path,
        }
    }
    fn internal_commit(&self) -> std::io::Result<()> {
        rename(&self.working_path, &self.target_path)
    }
    // rmv pub fn retry_strategy(&mut self, value: RetryStrategyRmv) {
    // rmv     self.retry_strategy_rmv = value;
    // rmv }
    // rmv #[cfg(not(feature = "tokio"))]
    // rmv fn short_nap(tries: i32) {
    // rmv     std::thread::sleep(std::time::Duration::from_millis(tries as u64*125));
    // rmv }
}

impl Drop for Phazer {
    /// `drop` removes the working file if it still exists (if the Phazer was not committed).
    fn drop(&mut self) {
        let _ = remove_file(&self.working_path);
    }
}

impl std::fmt::Debug for Phazer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Phazer")
            .field("target_path", &self.target_path)
            .field("working_path", &self.working_path)
            .field("last_commit_error", &self.last_commit_error)
            .finish()
    }
}

impl std::fmt::Display for Phazer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if let Some(lce) = &self.last_commit_error {
            format!(
                "last commit for {} failed with error: {}",
                self.target_path.display(),
                lce
            )
        } else {
            if self.commit_tried {
                format!("commit for {} was successful", self.target_path.display())
            } else {
                format!(
                    "commit for {} has not yet been attempted",
                    self.target_path.display()
                )
            }
        };
        f.write_str(&text)
    }
}

impl std::error::Error for Phazer {}

// Return a serial number for this application to ensure the working filename is unique.
fn current_phazer_id() -> usize {
    static NEXT_PHAZER_ID: AtomicUsize = AtomicUsize::new(0);
    NEXT_PHAZER_ID.fetch_add(1, Ordering::Relaxed)
}
