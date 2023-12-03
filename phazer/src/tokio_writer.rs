#![cfg(feature = "tokio")]
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

use std::marker::PhantomData;
use std::pin::Pin;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

impl Phazer {
    /// Returns an asynchronous file-like thing that's used to build the working file.
    ///
    /// This method is available when then `tokio` feature is enabled.
    pub async fn tokio_writer<'a>(&'a self) -> std::io::Result<TokioPhazerWriter> {
        let mut options = OpenOptions::new();
        // Always allow read / write
        options.read(true).write(true);
        // First pass?  Create and truncate.
        if !self.file_created.get() {
            self.file_created.set(true);
            options.truncate(true).create(true);
        };
        // Try to open / create the file
        let phase1 = options.open(&self.working_path).await?;
        Ok(TokioPhazerWriter {
            phase1,
            _parent: PhantomData::<&'a Self>,
        })
    }
}

/// TokioPhazerWriter is an asynchronous file-like thing that's used to build the working file.
///
/// It maintains a reference the the Phazer used to construct it, ensuring Phazer::commit cannot be
/// called if there are any potential writers.
///
/// This struct is available when the `tokio` feature is enabled.
pub struct TokioPhazerWriter<'a> {
    phase1: File,
    _parent: PhantomData<&'a Phazer>,
}

impl<'a> AsyncRead for TokioPhazerWriter<'a> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().poll_read(cx, buf)
    }
}

impl<'a> AsyncSeek for TokioPhazerWriter<'a> {
    fn poll_complete(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().poll_complete(cx)
    }
    fn start_seek(mut self: Pin<&mut Self>, position: std::io::SeekFrom) -> std::io::Result<()> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().start_seek(position)
    }
}

impl<'a> AsyncWrite for TokioPhazerWriter<'a> {
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().poll_shutdown(cx)
    }
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let mut pp: Pin<Box<&mut File>> = Pin::from(Box::new(&mut self.phase1));
        pp.as_mut().poll_write(cx, buf)
    }
}
