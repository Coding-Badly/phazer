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

use std::fs::{canonicalize, create_dir, metadata, set_permissions};
use std::io::Write;
use std::path::Path;

use rand::RngCore;

use phazer::Phazer;

fn write_test_file<F, R>(mut f: F, rng: &mut R) -> Result<(), std::io::Error>
where
    F: Write,
    R: RngCore,
{
    let mut s = rng.next_u32() & ((1024 * 1024) - 1);
    f.write_all(&s.to_be_bytes())?;
    let mut v = rng.next_u32();
    f.write_all(&v.to_be_bytes())?;
    while s > 0 {
        let b: [u8; 1] = [v as u8];
        f.write_all(&b)?;
        v += 1;
        s -= 1;
    }
    Ok(())
}

fn create_test_file<R>(path: &Path, rng: &mut R) -> Result<(), Box<dyn std::error::Error>>
where
    R: RngCore,
{
    let p = Phazer::new(path);
    let w = p.simple_writer()?;
    write_test_file(w, rng)?;
    p.commit()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();

    // Prepare various paths
    let working_dir = canonicalize(".")?;
    let local_dir = working_dir.join("local");
    let target_path = local_dir.join("readonly-test.bin");

    println!("Create the working directory {}...", local_dir.display());
    let _ = create_dir(&local_dir);

    println!("Create the test file {}...", target_path.display());
    let mut rng = rand::thread_rng();
    create_test_file(target_path.as_path(), &mut rng)?;

    println!("Make the test file {} read-only...", target_path.display());
    let m = metadata(&target_path)?;
    let mut p = m.permissions();
    p.set_readonly(true);
    set_permissions(&target_path, p)?;

    println!();
    Ok(())
}
