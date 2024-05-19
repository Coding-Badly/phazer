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

mod common;

#[cfg(feature = "simple")]
mod simple {
    use std::fs::{metadata, remove_file, set_permissions};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use phazer::Phazer;

    use crate::common::prepare_target_file;

    fn set_readonly<P: AsRef<Path>>(path: P, value: bool) -> Result<bool, std::io::Error> {
        let path = path.as_ref();
        let m =  metadata(path)?;
        let mut p = m.permissions();
        let rv = p.readonly();
        p.set_readonly(value);
        set_permissions(path, p)?;
        Ok(rv)
    }

    fn using_simple_rename<C, P>(
        phazer_new: C,
        filename: P,
    ) -> Result<Result<(), std::io::Error>, Box<dyn std::error::Error>>
    where
        C: Fn(&PathBuf) -> Phazer,
        P: AsRef<Path>,
    {
        let target_path = prepare_target_file(filename)?;

        let p = phazer_new(&target_path);
        let mut w = p.simple_writer()?;
        w.write_all("read-only".as_bytes())?;
        drop(w);
        p.commit()?;

        let orov = set_readonly(&target_path, true)?;

        let p = phazer_new(&target_path);
        let mut w = p.simple_writer()?;
        w.write_all("new stuff".as_bytes())?;
        drop(w);
        let rv = p.commit();

        set_readonly(&target_path, orov)?;

        let _ = remove_file(&target_path);

        Ok(rv)
    }

    #[test]
    fn junk() -> Result<(), Box<dyn std::error::Error>> {
        let rv = using_simple_rename(
            |p| Phazer::new(p),
            "junk.txt")?;
        println!("rv = {:?}", rv);

        #[cfg(unix)]
        match rv {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }

        #[cfg(windows)]
        match rv {
            Ok(()) => Err(std::io::Error::new(ErrorKind::Other, "windows is expected to fail").into()),
            Err(e) => {
                if e.kind() == ErrorKind::PermissionDenied {
                    Ok(())
                } else {
                    Err(e.into())
                }
            }
        }
    }
}

#[cfg(feature = "tokio")]
mod tokio {
}
