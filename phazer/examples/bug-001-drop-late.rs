#[cfg(feature = "simple")]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(not(feature = "simple"))]
mod inner {
    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!("This example requires the 'simple' feature to be enabled.  Try...");
        println!("cargo run --example download --features simple");
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::main()
}
