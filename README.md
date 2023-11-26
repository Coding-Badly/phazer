# Introduction

Welcome to the _phazer_ crate!

Imagine, if you will, that you are building an application that downloads a file from a website.
Let's say that the application is downloading the baby name data from the U.S. Social Security
Administration (https://www.ssa.gov/oact/babynames/names.zip).

A common failure when getting data from the internet is an interrupted download.  Unless
precautions are taken the file ends up truncated (essentially corrupt).  That would result in a
bad experience the user.  The application might stop running after outputting a cryptic error
regarding an unreadable ZIP file.

A similar problem occurs with configuration files.  We want our service to only see a complete
configuration file.  A partial configuration file might even introduce a security vulnerablility.

The purpose of this crate is to present a file to a system in a finished state or not at all.
Either the entire names.zip file is downloaded or the file is missing.  Either the old complete
configuration file is used or the new complete configuration file is used.

## License

_phazer_ is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

## Build Status

![Clippy](https://github.com/Coding-Badly/phazer/actions/workflows/clippy.yml/badge.svg)
![Examples](https://github.com/Coding-Badly/phazer/actions/workflows/examples.yml/badge.svg)
![Miri](https://github.com/Coding-Badly/phazer/actions/workflows/miri.yml/badge.svg)
![Test](https://github.com/Coding-Badly/phazer/actions/workflows/test.yml/badge.svg)
