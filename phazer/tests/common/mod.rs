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

#![allow(dead_code)]

mod simple_fs;

#[allow(unused_imports)]
pub use simple_fs::{prepare_target_file, prepare_working_dir};

// Used in no-writer-commit-works
pub const NO_WRITER_COMMIT_DEFAULT: &str = "no-writer-commit-default.txt";
pub const NO_WRITER_COMMIT_SIMPLE_RENAME: &str = "no-writer-commit-simple-rename.txt";
pub const NO_WRITER_COMMIT_WITH_RETRY: &str = "no-writer-commit-with-retry.txt";

// Used in one-wins-in-race
pub const ONE_WINS_IN_RACE_SIMPLE_RENAME: &str = "one-wins-in-race-simple-rename.txt";
pub const ONE_WINS_IN_RACE_SIMPLE_WITH_RETRY: &str = "one-wins-in-race-simple-with-retry.txt";
pub const ONE_WINS_IN_RACE_TOKIO_RENAME: &str = "one-wins-in-race-tokio-rename.txt";
pub const ONE_WINS_IN_RACE_TOKIO_WITH_RETRY: &str = "one-wins-in-race-tokio-with-retry.txt";

// Used in posix-read-only-works
pub const POSIX_READ_ONLY_DEFAULT: &str = "posix-read-only-default.txt";

// Used in write-commit-works
pub const WRITE_COMMIT_SIMPLE_DEFAULT: &str = "write-commit-simple-default.txt";
pub const WRITE_COMMIT_SIMPLE_RENAME: &str = "write-commit-simple-rename.txt";
pub const WRITE_COMMIT_SIMPLE_WITH_RETRY: &str = "write-commit-simple-with-retry.txt";
pub const WRITE_COMMIT_TOKIO_DEFAULT: &str = "write-commit-tokio-default.txt";
pub const WRITE_COMMIT_TOKIO_RENAME: &str = "write-commit-tokio-rename.txt";
pub const WRITE_COMMIT_TOKIO_WITH_RETRY: &str = "write-commit-tokio-with-retry.txt";

// Used in write-no-commit-works
pub const WRITE_NO_COMMIT_NO_TARGET_SIMPLE_DEFAULT: &str =
    "write-no-commit-no-target-simple-default.txt";
pub const WRITE_NO_COMMIT_NO_TARGET_SIMPLE_RENAME: &str =
    "write-no-commit-no-target-simple-rename.txt";
pub const WRITE_NO_COMMIT_NO_TARGET_SIMPLE_WITH_RETRY: &str =
    "write-no-commit-no-target-simple-with-retry.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_SIMPLE_DEFAULT: &str =
    "write-no-commit-have-target-simple-default.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_SIMPLE_RENAME: &str =
    "write-no-commit-have-target-simple-rename.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_SIMPLE_WITH_RETRY: &str =
    "write-no-commit-have-target-simple-with-retry.txt";
pub const WRITE_NO_COMMIT_NO_TARGET_TOKIO_DEFAULT: &str =
    "write-no-commit-no-target-tokio-default.txt";
pub const WRITE_NO_COMMIT_NO_TARGET_TOKIO_RENAME: &str =
    "write-no-commit-no-target-tokio-rename.txt";
pub const WRITE_NO_COMMIT_NO_TARGET_TOKIO_WITH_RETRY: &str =
    "write-no-commit-no-target-tokio-with-retry.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_TOKIO_DEFAULT: &str =
    "write-no-commit-have-target-tokio-default.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_TOKIO_RENAME: &str =
    "write-no-commit-have-target-tokio-rename.txt";
pub const WRITE_NO_COMMIT_HAVE_TARGET_TOKIO_WITH_RETRY: &str =
    "write-no-commit-have-target-tokio-with-retry.txt";
