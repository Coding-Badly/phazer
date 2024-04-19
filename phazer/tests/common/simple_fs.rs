use std::fs::{canonicalize, create_dir, remove_file};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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

pub fn prepare_working_dir() -> Result<PathBuf, std::io::Error> {
    let mut working_dir = canonicalize(".")?;
    working_dir.push("local");
    ignore(create_dir(&working_dir), ErrorKind::AlreadyExists)?;
    Ok(working_dir)
}

fn _prepare_target_file(filename: &Path) -> Result<PathBuf, std::io::Error> {
    let working_dir = prepare_working_dir()?;
    let target_path = working_dir.join(filename);
    ignore(remove_file(&target_path), ErrorKind::NotFound)?;
    Ok(target_path)
}

#[allow(unused)]
pub fn prepare_target_file<P>(filename: P) -> Result<PathBuf, std::io::Error>
where
    P: AsRef<Path>,
{
    _prepare_target_file(filename.as_ref())
}
