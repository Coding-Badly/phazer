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

#[cfg(all(windows,feature = "simple"))]
mod inner {
    use std::fs::{canonicalize, metadata, set_permissions};
    use std::io::{ErrorKind, Write};
    use std::path::Path;

    use phazer::Phazer;

    fn set_readonly<P: AsRef<Path>>(path: P, value: bool) -> Result<bool, std::io::Error> {
        let path = path.as_ref();
        let m = metadata(path)?;
        let mut p = m.permissions();
        let rv = p.readonly();
        p.set_readonly(value);
        set_permissions(path, p)?;
        Ok(rv)
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Use a full path so we can freely change the working directory
        let full_path = canonicalize("read-only-windows.txt")?;

        // Create the target
        let phazer = Phazer::new(&full_path);
        let mut writer = phazer.simple_writer()?;
        writer.write_all("read-only".as_bytes())?;
        drop(writer);
        phazer.commit()?;

        // Set the read-only attribute
        set_readonly(&full_path, true)?;

        // Try to update it...
        let phazer = Phazer::new(&full_path);
        let mut writer = phazer.simple_writer()?;
        writer.write_all("something new".as_bytes())?;
        drop(writer);
        // ...using commit2 so we can recover if the read-only attribute is set.
        match phazer.commit2() {
            Ok(()) => {
                println!("Success!  This was unexpected.  Windows up to 11 always failed.");
            }
            Err((e, p)) => {
                // If the error is anything except PermissionDenied then return it
                if e.kind() != ErrorKind::PermissionDenied {
                    return Err(e.into());
                }
                // Clear the read-only attribute
                let _ = set_readonly(&full_path, false);
                // Try again
                p.commit()?;
            }
        }
        Ok(())
    }
}

#[cfg(all(windows,not(feature = "simple")))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!("This example requires the 'simple' feature to be enabled.  Try...");
        println!("cargo run --example read-only-windows --features simple");
        println!();
        Ok(())
    }
}

#[cfg(not(windows))]
mod innner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!("This example is only available for Windows.");
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
