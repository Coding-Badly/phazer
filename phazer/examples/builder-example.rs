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

#[cfg(unix)]
mod os {
    use phazer::{Phazer, PhazerBuilder, SIMPLE_RENAME_STRATEGY};

    pub fn build_phazer() -> Phazer<'static> {
        PhazerBuilder::with_target("fight-for-it.txt")
            .commit_strategy(SIMPLE_RENAME_STRATEGY)
            .build()
    }
}

#[cfg(windows)]
mod os {
    use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY};

    pub fn build_phazer() -> Phazer<'static> {
        PhazerBuilder::new()
            .commit_strategy(RENAME_WITH_RETRY_STRATEGY)
            .target("fight-for-it.txt")
            .build()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = os::build_phazer();

    #[cfg(feature = "simple")]
    {
        use std::io::Write;

        let mut w = p.simple_writer()?;
        w.write_all("first".as_bytes())?;
        drop(w);
    }

    p.commit()?;

    Ok(())
}
