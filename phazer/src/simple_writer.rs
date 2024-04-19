#![cfg(feature = "simple")]
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

use crate::Phazer;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::marker::PhantomData;

impl<'cs> Phazer<'cs> {
    /// Returns a synchronous file-like thing that's used to build the working file.
    ///
    /// This method is available when the `simple` feature is enabled.
    pub fn simple_writer<'a>(&'a self) -> std::io::Result<SimplePhazerWriter> {
        let mut options = OpenOptions::new();
        // Always allow read / write
        options.read(true).write(true);
        // Is this the first writer?  Create and truncate.
        if self.first_writer() {
            options.truncate(true).create(true);
        }
        // Try to open / create the file
        let phase1 = options.open(&self.working_path)?;
        Ok(SimplePhazerWriter {
            phase1,
            _parent: PhantomData::<&'a Self>,
        })
    }
}

/// SimplePhazerWriter is a synchronous file-like thing that's used to build the working file.
///
/// It maintains a reference to the Phazer used to construct it, ensuring Phazer::commit cannot be
/// called if there are any writers.
///
/// This struct is available when the `simple` feature is enabled.
pub struct SimplePhazerWriter<'a, 'cs> {
    phase1: File,
    _parent: PhantomData<&'a Phazer<'cs>>,
}

impl<'p, 'cs> Drop for SimplePhazerWriter<'p, 'cs> {
    fn drop(&mut self) {}
}

impl<'a, 'cs> Read for SimplePhazerWriter<'a, 'cs> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.phase1.read(buf)
    }
}

impl<'a, 'cs> Seek for SimplePhazerWriter<'a, 'cs> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.phase1.seek(pos)
    }
}

impl<'a, 'cs> Write for SimplePhazerWriter<'a, 'cs> {
    fn flush(&mut self) -> std::io::Result<()> {
        self.phase1.flush()
    }
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.phase1.write(buf)
    }
}
