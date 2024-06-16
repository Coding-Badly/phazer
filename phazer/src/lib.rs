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

//! Imagine, if you will, building an application that downloads a file from a website.  Let's say
//! the application is downloading baby name data from the U.S. Social Security
//! Administration (<https://www.ssa.gov/oact/babynames/names.zip>).
//!
//! A common failure when getting data from the internet is an interrupted download.  Unless
//! precautions are taken, the file ends up truncated (essentially corrupt).  That could easily
//! result in a bad experience.  The application might stop running after outputting a cryptic error
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
//! # Glossary
//!
//! Two important terms are used throughout this documentation...
//! * target - This is the "final" file.  Continuing from the earlier examples, this would be the
//! downloaded file when it has been successfully downloaded or the new configuration file when it's
//! ready to be used.
//! * working - This is the "temporary" file.  Writing is to the working file.  This crate manages
//! the working file including generating a unique filename and discarding the file if it is not
//! committed.
//!
//! # Getting Started
//!
//! [`Phazer`] is the core component of this crate.  One [`Phazer`] is constructed for each target
//! file.  [`Phazer`] is essentially a wrapper over the target path.  For example, if the
//! application downloads three files from the internet then one [`Phazer`] is created for each
//! file.
//!
//! [`Phazer`] provides a convenient way to manage the working file.
//! When working with the Rust Standard Library, the [`simple_writer`][sw] method returns a
//! [`SimplePhazerWriter`][spw] which provides read / write access to the working file.
//! When working with the [tokio crate][tc], the [`tokio_writer`][tw] method
//! returns a [`TokioPhazerWriter`][tpw] which provides async read / write access to the working file.
//!
//! [tc]: https://crates.io/crates/tokio
//! [sw]: crate::Phazer::simple_writer
//! [spw]: crate::simple_writer::SimplePhazerWriter
//! [tw]: crate::Phazer::tokio_writer
//! [tpw]: crate::tokio_writer::TokioPhazerWriter
//!
//! By default, [`Phazer`] uses a simple rename commit strategy ([`SIMPLE_RENAME_STRATEGY`]).  When
//! [`Phazer::commit`] is called, [`rename`] is used to replace the target file with the working
//! file.  [`PhazerBuilder`] can be used to construct a [`Phazer`] with a different commit strategy.
//! The one other commit strategy available with this crate is [`RENAME_WITH_RETRY_STRATEGY`].
//!

pub mod simple_writer;
pub mod tokio_writer;

use std::fs::{remove_file, rename};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[doc(hidden)]
pub trait CommitDetails {
    fn get_working_path(&self) -> &Path;
    fn get_target_path(&self) -> &Path;
    fn get_jitter(&self) -> usize;
}

#[doc(hidden)]
pub trait CommitStrategy: Sync {
    fn commit(&self, phazer: &dyn CommitDetails) -> std::io::Result<()>;
}

/// [`Phazer`] manages the transition of the working file to the target file.
///
/// [`Phazer`] is the core component of this crate.  One [`Phazer`] is constructed for each target
/// file.  [`Phazer`] essentially a wrapper over the target path.  For example, if the application
/// downloads three files from the internet then one [`Phazer`] is created for each file.
///
/// By default, [`Phazer`] uses a simple rename commit strategy ([`SIMPLE_RENAME_STRATEGY`]).  When
/// [`Phazer::commit`] is called, [`rename`] is used to replace the target file with the working
/// file.  [`PhazerBuilder`] can be used to construct a [`Phazer`] with a different commit strategy.
/// The one other commit strategy available with this crate is [`RENAME_WITH_RETRY_STRATEGY`].
///
pub struct Phazer<'cs> {
    file_created: AtomicBool,
    commit_strategy: &'cs dyn CommitStrategy,
    working_path: PathBuf,
    target_path: PathBuf,
    phazer_id: usize,
}

impl<'cs> Phazer<'cs> {
    /// Creates a [`Phazer`] where `path` is the target file.
    ///
    /// # Arguments
    ///
    /// * `path` - Target file.  Ideally, the full path is specified so changes to the working
    /// directory do not cause problems.  [canonicalize][std::fs::canonicalize] is helpful.
    ///
    /// # Return Value
    ///
    /// A new [`Phazer`] is always returned; [`Phazer::new`] is infallible.
    ///
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
    /// [`commit`][pc] transfers the working file to the target file; by default this is done with
    /// a [rename].
    ///
    /// If the working file was not created then [`commit`][pc] simply returns `Ok(())`.
    ///
    /// The working file cannot be open when [`commit`][pc] is called.  This is enforced by a
    /// lifetime connecting each writer to the [`Phazer`] that created it.  If [`commit`][pc] is
    /// called when a writer is active then an error similar to the following occurs when
    /// compiling...
    ///
    /// &nbsp;&nbsp;&nbsp;&nbsp;`error[E0505]: cannot move out of 'phazer' because it is borrowed`
    ///
    /// Often, writers must be explicitly dropped before calling [`commit`][pc].  This is shown in
    /// the example.
    ///
    /// [pc]: Phazer::commit
    ///
    /// # Return Value
    ///
    /// An [`Error`][ioe] is returned if the working file cannot be transferred to the target file.
    /// In this case the working file is removed.
    ///
    /// [ioe]: std::io::Error
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "simple")]
    /// # {
    /// use std::fs::canonicalize;
    /// use std::io::Write;
    ///
    /// use phazer::Phazer;
    ///
    /// pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Use a full path so we can freely change the working directory
    ///     let target = canonicalize("config.toml")?;
    ///     // Create the Phazer
    ///     let phazer = Phazer::new(&target);
    ///     // Write some stuff.  Drop the writer to ensure the file is not open.
    ///     let mut writer = phazer.simple_writer()?;
    ///     writer.write_all("[Serial Port]\nbaud = 250000\n".as_bytes())?;
    ///
    ///     // Note the explicit `drop`.  This ensures `writer` does not exist when `commit` is
    ///     // called.  That the working file is no longer open.
    ///     drop(writer);
    ///
    ///     // Rename the working file to the target file ("save" the changes)
    ///     phazer.commit()?;
    ///     Ok(())
    /// }
    /// # }
    /// ```
    ///
    pub fn commit(self) -> Result<(), std::io::Error> {
        self.commit2().map_err(|e| e.0)
    }

    /// [`commit2`][pc] transfers the working file to the target file; by default this is done with
    /// a [rename].
    ///
    /// If the working file was not created then [`commit2`][pc] simply returns `Ok(())`.
    ///
    /// The working file cannot be open when [`commit2`][pc] is called.  This is enforced by a
    /// lifetime connecting each writer to the [`Phazer`] that created it.  If [`commit2`][pc] is
    /// called when a writer is active then an error similar to the following occurs when
    /// compiling...
    ///
    /// &nbsp;&nbsp;&nbsp;&nbsp;`error[E0505]: cannot move out of 'phazer' because it is borrowed`
    ///
    /// Often, writers must be explicitly dropped before calling [`commit2`][pc].  This is shown in
    /// the example.
    ///
    /// [pc]: Phazer::commit2
    ///
    /// # Return Value
    ///
    /// An [`Error`][ioe] and the [`Phazer`] are returned if the working file cannot be transferred
    /// to the target file.  This allows for error recovery not provided by this crate.  For
    /// example, on Windows, a target file with the read-only attribute set cannot be replaced with
    /// a [`rename`].  This is demonstrated in the example.
    ///
    /// [ioe]: std::io::Error
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "simple")]
    /// # {
    /// use std::fs::{canonicalize, metadata, set_permissions};
    /// use std::io::{ErrorKind, Write};
    /// use std::path::Path;
    ///
    /// use phazer::Phazer;
    ///
    /// fn set_readonly<P: AsRef<Path>>(path: P, value: bool) -> Result<bool, std::io::Error> {
    ///     let path = path.as_ref();
    ///     let m = metadata(path)?;
    ///     let mut p = m.permissions();
    ///     let rv = p.readonly();
    ///     p.set_readonly(value);
    ///     set_permissions(path, p)?;
    ///     Ok(rv)
    /// }
    ///
    /// pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Use a full path so we can freely change the working directory
    ///     let target = canonicalize("read-only-windows.txt")?;
    ///
    ///     // Create the target
    ///     let phazer = Phazer::new(&target);
    ///     let mut writer = phazer.simple_writer()?;
    ///     writer.write_all("read-only".as_bytes())?;
    ///     drop(writer);
    ///     phazer.commit()?;
    ///
    ///     // Set the read-only attribute
    ///     set_readonly(&target, true)?;
    ///
    ///     // Try to update it...
    ///     let phazer = Phazer::new(&target);
    ///     let mut writer = phazer.simple_writer()?;
    ///     writer.write_all("something new".as_bytes())?;
    ///     drop(writer);
    ///     // ...using commit2 so we can recover if the read-only attribute is set.
    ///     match phazer.commit2() {
    ///         Ok(()) => {
    ///             println!("Success!  This was unexpected.  Windows up to 11 always failed.");
    ///         }
    ///         Err((e, p)) => {
    ///             // If the error is anything except PermissionDenied then return it
    ///             if e.kind() != ErrorKind::PermissionDenied {
    ///                 return Err(e.into());
    ///             }
    ///             // Clear the read-only attribute
    ///             let _ = set_readonly(&target, false);
    ///             // Try again
    ///             p.commit()?;
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// # }
    /// ```
    ///
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
    #[doc(hidden)]
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

/// [`SimpleRenameStrategy`] uses the Standard Library [`rename`] function to transition the working
/// file to the target file.
///
/// The other commit strategy available is [`RenameWithRetryStrategy`].
///
/// For POSIX systems and Windows systems in which there is no contention for the target file,
/// [`SimpleRenameStrategy`] is a good choice.  For Windows systems in which two or more threads are
/// simultaneously trying to update the target, [`RenameWithRetryStrategy`] is a good choice.
///
/// This crate provides a ready-to-use [`SimpleRenameStrategy`] instance named
/// [`SIMPLE_RENAME_STRATEGY`].
///
/// By default, this commit strategy is used.
///
/// # Example
///
/// ```
/// use phazer::{PhazerBuilder, SIMPLE_RENAME_STRATEGY};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let phazer = PhazerBuilder::with_target("uses-simple-rename-strategy.txt")
///         .commit_strategy(SIMPLE_RENAME_STRATEGY)
///         .build();
///
///     // Build the working file
///
///     // `rename` is called to transition the working file to the target
///     phazer.commit()?;
///
///     Ok(())
/// }
/// ```
///

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

/// A ready-to-use instance of [`SimpleRenameStrategy`].
pub const SIMPLE_RENAME_STRATEGY: &dyn CommitStrategy = &SimpleRenameStrategy {};

/// [`RenameWithRetryStrategy`] uses the Standard Library [`rename`] function to transition the
/// working file to the target file and retries if that fails with a [`PermissionDenied`][pd] error.
///
/// The other commit strategy available is [`SimpleRenameStrategy`].
///
/// For POSIX systems and Windows systems in which there is no contention for the target file,
/// [`SimpleRenameStrategy`] is a good choice.  For Windows systems in which two or more threads are
/// simultaneously trying to update the target, [`RenameWithRetryStrategy`] is a good choice.
///
/// This crate provides a ready-to-use [`RenameWithRetryStrategy`] instance named
/// [`RENAME_WITH_RETRY_STRATEGY`].
///
/// By default, the [`SimpleRenameStrategy`] commit strategy is used.
///
/// [pd]: std::io::ErrorKind::PermissionDenied
///
/// This retry strategy has been shown to work well with Windows 10 and Windows 11 on local SSD
/// drives and with a NAS using as many as 10 threads contending for the target file...
/// * Use a "jitter" between 0 and 15 to avoid threads being synchronized during contention
/// * Calculate a "base sleep" value: 11 + (3 * jitter)
/// * Try to commit
/// * If that succeeds then we're done
/// * If that fails with any error except [`PermissionDenied`][pd] then return that error
/// * Otherwise sleep for the base sleep value multiplied by the try count.  For example...
///     * If the jitter is 1
///     * Then the base sleep is 11 + (3 * 1) = 14
///     * For the first try, this strategy would sleep for 14 * 1 milliseconds
///     * For the second try, this strategy would sleep for 14 * 2 milleconds
/// * Until 7 attempts have been made at which point the [`PermissionDenied`][pd] error is returned.
///
/// In the worst case, this strategy sleeps for a total of (11 + (3 * 15)) * (7 * (7+1) / 2) = 1568 milliseconds.
///
/// # Example
///
/// ```
/// use phazer::{PhazerBuilder, RENAME_WITH_RETRY_STRATEGY};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let phazer = PhazerBuilder::with_target("uses-rename-with-retry-strategy.txt")
///         .commit_strategy(RENAME_WITH_RETRY_STRATEGY)
///         .build();
///
///     // Build the working file
///
///     // `rename` is called to transition the working file to the target
///     phazer.commit()?;
///
///     Ok(())
/// }
/// ```
///
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

/// A ready-to-use instance of [`RenameWithRetryStrategy`].
pub const RENAME_WITH_RETRY_STRATEGY: &dyn CommitStrategy = &RenameWithRetryStrategy {};

#[doc = include_str!("doc/phazer-builder-overview.md")]
pub struct PhazerBuilder<'cs> {
    commit_strategy: Option<&'cs dyn CommitStrategy>,
}

#[doc = include_str!("doc/phazer-builder-overview.md")]
pub struct PhazerBuilderWithTarget<'cs> {
    commit_strategy: Option<&'cs dyn CommitStrategy>,
    target_path: PathBuf,
}

impl<'cs> PhazerBuilder<'cs> {
    /// Creates an uninitialized [`PhazerBuilder`].
    ///
    /// The target path must be set (which converts a [`PhazerBuilder`] into a
    /// [`PhazerBuilderWithTarget`]) before a [`Phazer`] can be built.
    ///
    /// By default a simple rename commit strategy ([`SIMPLE_RENAME_STRATEGY`]) is used.
    ///
    /// # Return Value
    ///
    /// A new [`PhazerBuilder`] is always returned; [`PhazerBuilder::new`] is infallible.
    ///
    pub fn new() -> Self {
        Self {
            commit_strategy: None,
        }
    }
    /// Creates a [`PhazerBuilderWithTarget`].
    ///
    /// With the target path set, a [`Phazer`] can be built with the default commit strategy
    /// ([`SIMPLE_RENAME_STRATEGY`]) or a different commit strategy can be used with a call to
    /// [`PhazerBuilderWithTarget::commit_strategy`].
    ///
    /// # Arguments
    ///
    /// * `path` - Target file.  Ideally, the full path is specified so changes to the working
    /// directory do not cause problems.  [canonicalize][std::fs::canonicalize] is helpful.
    ///
    /// # Return Value
    ///
    /// A new [`PhazerBuilderWithTarget`] is always returned; [`PhazerBuilder::with_target`] is
    /// infallible.
    ///
    pub fn with_target<P>(path: P) -> PhazerBuilderWithTarget<'cs>
    where
        P: Into<PathBuf>,
    {
        PhazerBuilderWithTarget {
            commit_strategy: None,
            target_path: path.into(),
        }
    }
    /// Converts a [`PhazerBuilder`] to a [`PhazerBuilderWithTarget`] by adding the specified target
    /// path.
    ///
    /// If a commit strategy was specified in a previous call to [`PhazerBuilder::commit_strategy`],
    /// that strategy is passed to the returned [`PhazerBuilderWithTarget`].
    ///
    /// # Arguments
    ///
    /// * `value` - Target file.  Ideally, the full path is specified so changes to the working
    /// directory do not cause problems.  [canonicalize][std::fs::canonicalize] is helpful.
    ///
    /// # Return Value
    ///
    /// A new [`PhazerBuilderWithTarget`] is always returned; [`PhazerBuilder::target`] is infallible.
    ///
    pub fn target<P>(self, value: P) -> PhazerBuilderWithTarget<'cs>
    where
        P: Into<PathBuf>,
    {
        PhazerBuilderWithTarget {
            commit_strategy: self.commit_strategy,
            target_path: value.into(),
        }
    }
    /// Changes the commit strategy the [`Phazer`] uses when [`commit`][pc] is called.
    ///
    /// The default commit strategy ([`SIMPLE_RENAME_STRATEGY`]) is used if a strategy is never
    /// assigned.  This crate provides one other strategy ([`RENAME_WITH_RETRY_STRATEGY`]).
    ///
    /// # Arguments
    ///
    /// * `value` - The commit strategy that's used by the created [`Phazer`].
    ///
    /// [pc]: crate::Phazer::commit
    ///
    pub fn commit_strategy(mut self, value: &'cs dyn CommitStrategy) -> Self {
        self.commit_strategy = Some(value);
        self
    }
}

impl<'cs> PhazerBuilderWithTarget<'cs> {
    /// Changes the target path.
    ///
    /// # Arguments
    ///
    /// * `value` - Target file.  Ideally, the full path is specified so changes to the working
    /// directory do not cause problems.  [canonicalize][std::fs::canonicalize] is helpful.
    ///
    pub fn target<P>(mut self, value: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.target_path = value.into();
        self
    }
    /// Changes the commit strategy the [`Phazer`] uses when [`commit`][pc] is called.
    ///
    /// The default commit strategy ([`SIMPLE_RENAME_STRATEGY`]) is used if a strategy is never
    /// assigned.  This crate provides one other strategy ([`RENAME_WITH_RETRY_STRATEGY`]).
    ///
    /// # Arguments
    ///
    /// * `value` - The commit strategy that's used by the created [`Phazer`].
    ///
    /// [pc]: crate::Phazer::commit
    ///
    pub fn commit_strategy(mut self, value: &'cs dyn CommitStrategy) -> Self {
        self.commit_strategy = Some(value);
        self
    }
    /// Builds a new [`Phazer`] using the target path and commit strategy.
    ///
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
