
mod inner {
    use std::fs::{create_dir, metadata, remove_file, set_permissions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use phazer::{Never, Phazer};
    use rand::RngCore;

    fn get_test_path() -> &'static PathBuf {
        static NAMES_PATH: OnceLock<PathBuf> = OnceLock::new();
        NAMES_PATH.get_or_init(|| {
            PathBuf::from("local/readonly-test.bin")
        })
    }
    
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

    fn create_test_file<R>(rng: &mut R) -> Result<(), Box<dyn std::error::Error>>
    where
        R: RngCore,
    {
        let p = Phazer::new(get_test_path(), Never::default());
        let w = p.simple_writer()?;
        write_test_file(w, rng)?;
        p.commit()?;
        Ok(())
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        println!("Create a directory for the test file...");
        let _ = create_dir("local");
        println!("Create the initial test file...");
        create_test_file(&mut rng)?;
        println!("Make the test file read-only...");
        let mut p = metadata(get_test_path())?.permissions();
        p.set_readonly(true);
        set_permissions(get_test_path(), p)?;
        println!("Try writing a new file...");
        match create_test_file(&mut rng) {
            Ok(()) => println!("Success."),
            Err(e) => println!("Failed with: {:?}, {}", e, e),
        }
        println!("Make the test file read-write...");
        let mut p = metadata(get_test_path())?.permissions();
        p.set_readonly(false);
        set_permissions(get_test_path(), p)?;
        println!("Remove the test file...");
        remove_file(get_test_path())?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
