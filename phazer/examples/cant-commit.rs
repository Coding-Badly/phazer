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

#[cfg(feature = "simple")]
mod inner {
    use phazer::Phazer;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let phazer = Phazer::new("cant-commit.txt");
        let _writer = phazer.simple_writer()?;
        // phazer.commit()?;
        // error[E0505]: cannot move out of `phazer` because it is borrowed
        Ok(())
    }
}

#[cfg(not(feature = "simple"))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
