#[cfg(feature = "simple")]
mod inner {
    use std::fs::{create_dir, remove_dir, remove_file, File, OpenOptions};
    use std::io::{Read, Write};

    use phazer::{Never, Phazer};
    use rand::RngCore;

    fn check_test_file() -> Result<File, std::io::Error> {
        let mut f = OpenOptions::new()
            .read(true)
            .open("local/conflict-test.bin")?;
        let mut b4 = [0u8, 0, 0, 0];
        f.read_exact(&mut b4[..])?;
        let mut s = u32::from_be_bytes(b4);
        f.read_exact(&mut b4[..])?;
        let mut v = u32::from_be_bytes(b4);
        while s > 0 {
            let mut b = [0u8];
            f.read_exact(&mut b)?;
            if v as u8 != b[0] {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("value mismatch"),
                ));
            }
            v += 1;
            s -= 1;
        }
        Ok(f)
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
        let p = Phazer::new("local/conflict-test.bin", Never::default());
        let w = p.simple_writer()?;
        write_test_file(w, rng)?;
        p.commit()?;
        Ok(())
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();

        let mut rng = rand::thread_rng();
        println!("Create a directory for the test file...");
        let _ = create_dir("local");
        println!("Create the initial test file...");
        let _ = create_test_file(&mut rng)?;
        println!("Validate the test file; leave it open for the test...");
        let f = check_test_file()?;
        println!("Try writing a new file...");
        match create_test_file(&mut rng) {
            Ok(()) => println!("Success."),
            Err(e) => println!("Failed with: {:?}, {}", e, e),
        }
        // Ensure the initial test file is not closed until here
        drop(f);
        println!("Remove the test file...");
        //remove_file("local/conflict-test.bin")?;
        println!("Try removing the test directory...");
        //let _ = remove_dir("local");
        println!("Finished.");

        println!();
        Ok(())
    }
}

#[cfg(not(feature = "simple"))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!("This example requires the 'simple' feature to be enabled.  Try running...");
        println!("cargo run --example conflict --features simple");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
