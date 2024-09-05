#![deny(missing_docs)]
//!![docs.rs](https://img.shields.io/docsrs/libwebnovel)
//!
//! This crate deals with webnovels. You can see it as a way to access different
//! webnovel hosting sites and be able to get their contents.
//!
//! Since there are times we don't have Internet access, such as when riding
//! some trains, downloading to disk in a convenient format seems the way to go.
//!
//! ## Example
//! Say you want to create a software that will generate epubs from a given
//! fiction url. This could be expressed by something like the following:
//!
//! ```rust
//! # use std::fs::File;
//! # use std::io::Write;
//! # use std::io;
//! use libwebnovel::{Backend, Backends, Chapter};
//!
//! fn main() {
//!     // Get the backend matching the given URL
//!     let fiction_backend = Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
//!     // Get all the chapters of the webnovel
//!     let chapters = fiction_backend.get_chapters().unwrap();
//!
//!     // write the resulting epub
//!     let epub_path = format!("{}.epub", fiction_backend.title().unwrap());
//!     let mut f = File::create(&epub_path).unwrap();
//!     write_chapters_to_epub(&mut f, &chapters).unwrap();
//!
//!     // Since this code example also sort of serves as an integration test, remove the created file :p
//!     std::fs::remove_file(epub_path).unwrap();
//! }
//!
//! fn write_chapters_to_epub(writer: &impl Write, chapters: &[Chapter]) -> Result<(), io::Error> {
//!     // do stuff to create the ebook here
//!     Ok(())
//! }
//! ```
//!
//! ## TODO
//!
//! - [ ] Find a way to handle something other than text content:
//!  - [ ] images
//!  - [ ] tables
//!  - [ ] chapter headers ?
//!  - [ ] chapter footers ?
//! - [ ] Add more backends, such as libread.
//! - [ ] create a binary

use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters, Setters};

/// A chapter of a webnovel
#[derive(Debug, Getters, Setters, CopyGetters, Default)]
#[getset(get = "pub")]
pub struct Chapter {
    /// Index of this chapter in the grand scheme of things
    index: u32,
    /// Title of this chapter, if any
    title: Option<String>,
    /// Content of this chapter
    content: String,
    /// date this chapter was published
    published_at: Option<DateTime<Utc>>,
}

/// implementations of backends
pub mod backends;
pub use backends::{Backend, Backends};
