[`PhazerBuilder`] and [`PhazerBuilderWithTarget`] are used to create a customized [`Phazer`].

Currently, the only customization available is the commit strategy.  By default [`Phazer`]
uses a simple rename commit strategy ([`SIMPLE_RENAME_STRATEGY`]).  For Windows, when there is
contention for the target, the [`RENAME_WITH_RETRY_STRATEGY`] is a better choice.

# Example

```
#[cfg(not(windows))]
mod os {
    use phazer::{Phazer, PhazerBuilder, SIMPLE_RENAME_STRATEGY};

    // Build a Phazer for POSIX (not-Windows) systems.  `rename` is guaranteed to replace the
    // target.
    pub fn build_phazer() -> Phazer<'static> {
        PhazerBuilder::with_target("fight-for-it.txt")
            .commit_strategy(SIMPLE_RENAME_STRATEGY)
            .build()
    }
}

#[cfg(windows)]
mod os {
    use phazer::{Phazer, PhazerBuilder, RENAME_WITH_RETRY_STRATEGY};

    // Build a Phazer for Windows.  `rename` sometimes fails when there's contention for the
    // target.  Automatically trying again seems to work well.
    pub fn build_phazer() -> Phazer<'static> {
        PhazerBuilder::new()
            .commit_strategy(RENAME_WITH_RETRY_STRATEGY)
            .target("fight-for-it.txt")
            .build()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Build a Phazer appropriate for the operating system.
    let p = os::build_phazer();

    // Do something with that Phazer.
    #[cfg(feature="simple")]
    {
        use std::io::Write;
        let mut w = p.simple_writer()?;
        w.write_all("first".as_bytes())?;
        drop(w);
    }
    p.commit()?;

    Ok(())
}
```
