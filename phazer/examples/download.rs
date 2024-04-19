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

#[cfg(feature = "tokio")]
mod inner {
    use std::ffi::OsString;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use futures_util::StreamExt;
    use phazer::Phazer;
    use tokio::fs::{create_dir, read_dir, remove_file};
    use tokio::io::AsyncWriteExt;

    #[derive(Debug)]
    enum LocalError {
        BadNews1,
        BadNews2,
        BadNews3,
        BadNews4,
        BadNews5,
        DownloadFailed(url::Url, reqwest::StatusCode),
        Test1,
    }

    impl std::fmt::Display for LocalError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::BadNews1 => f.write_str("the first self-check failed"),
                Self::BadNews2 => f.write_str("the second self-check failed"),
                Self::BadNews3 => f.write_str("the third self-check failed"),
                Self::BadNews4 => f.write_str("the fourth self-check failed"),
                Self::BadNews5 => f.write_str("the fifth self-check failed"),
                Self::DownloadFailed(u, sc) => {
                    let text = format!(
                        "failed to download {}; the error message is {}",
                        u,
                        sc.as_str()
                    );
                    f.write_str(&text)
                }
                Self::Test1 => f.write_str("stop writing to test error recovery"),
            }
        }
    }

    impl std::error::Error for LocalError {}

    async fn try_download(limit: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Preparing the destination file...");
        let p = Phazer::new("downloads/names.zip");
        let mut dst = p.tokio_writer().await?;

        println!("Starting the download...");
        // https://users.rust-lang.org/t/async-download-very-large-files/79621/2?u=coding-badly
        // ...thank you @alice!
        let url = url::Url::parse("https://www.ssa.gov/oact/babynames/names.zip").unwrap();
        let response = reqwest::get(url.clone()).await?;
        if !response.status().is_success() {
            return Err(Box::new(LocalError::DownloadFailed(
                url.clone(),
                response.status(),
            )));
        }
        let mut src = response.bytes_stream();
        let mut total: u64 = 0;
        while let Some(chunk_result) = src.next().await {
            let chunk = chunk_result?;
            dst.write_all(&chunk).await?;
            let written = chunk.len() as u64;
            println!("  {} bytes written", written);
            total += written;
            if total >= limit {
                return Err(Box::new(LocalError::Test1));
            }
        }
        println!("Dealing with any stragglers...");
        dst.flush().await?;

        println!("Download finished.  Committing...");
        p.commit().map_err(|e| e.0)?;

        Ok(())
    }

    async fn remove_old_files<P>(start_here: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let mut removed = 0;
        let mut p = start_here.as_ref().to_path_buf();

        let mut trash: Vec<OsString> = Vec::new();
        p.push("names*");
        let mut rd = read_dir("downloads").await?;
        p.pop();
        while let Some(entry) = rd.next_entry().await? {
            trash.push(entry.file_name());
        }
        for entry in trash.into_iter() {
            p.push(entry);
            remove_file(&p).await?;
            removed += 1;
            p.pop();
        }
        Ok(removed)
    }

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

    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();

        let mut d = PathBuf::from("downloads");

        println!("Creating the 'downloads' directory...");
        ignore(create_dir(&d).await, ErrorKind::AlreadyExists)?;

        println!("Removing old files from the 'downloads' directory...");
        remove_old_files(&d).await?;

        println!("Download 1 MiB then fail...");
        match try_download(1024 * 1024).await {
            Ok(()) => return Err(Box::new(LocalError::BadNews1)),
            Err(e) => {
                let text = format!("{:?}", e);
                if text != "Test1" {
                    return Err(Box::new(LocalError::BadNews2));
                }
            }
        }
        if remove_old_files(&d).await? != 0 {
            return Err(Box::new(LocalError::BadNews3));
        }
        println!("All good.");

        println!("Download the complete file...");
        try_download(u64::MAX).await?;
        d.push("names.zip");
        if !d.exists() {
            return Err(Box::new(LocalError::BadNews4));
        }
        d.pop();
        if remove_old_files(&d).await? != 1 {
            return Err(Box::new(LocalError::BadNews5));
        }
        println!("Finished.");

        println!();
        Ok(())
    }
}

#[cfg(not(feature = "tokio"))]
mod inner {
    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!("This example requires the 'tokio' feature to be enabled.  Try...");
        println!("cargo run --example download --features tokio");
        println!();
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main().await
}
