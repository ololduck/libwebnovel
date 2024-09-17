#![deny(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::bare_urls
)]
#![warn(rustdoc::unescaped_backticks)]
//! [![docs.rs](https://img.shields.io/docsrs/libwebnovel)](https://docs.rs/libwebnovel)
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
//!     let fiction_backend =
//!         Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
//!     // Get all the chapters of the webnovel
//!     let chapters = fiction_backend.get_chapters().unwrap();
//!
//!     // write the resulting epub
//!     let epub_path = format!("{}.epub", fiction_backend.title().unwrap());
//!     let mut f = File::create(&epub_path).unwrap();
//!     write_chapters_to_epub(&mut f, &chapters).unwrap();
//!
//!     // Since this code example also sort of serves as an integration test,
//!     // remove the created file :p
//!     std::fs::remove_file(epub_path).unwrap();
//! }
//!
//! fn write_chapters_to_epub(writer: &impl Write, chapters: &[Chapter]) -> Result<(), io::Error> {
//!     // do stuff to create the ebook here
//!     Ok(())
//! }
//! ```
//!
//! See [`Backends`] for more information on how to use the library. The
//! documentation of the [`Backend`] trait may also be useful, especially if you
//! want to implement an other backend (don't forget to share it with the [main repository](https://codeberg.org/paulollivier/libwebnovel)!).
//!
//! ## Supported providers
//!
//! - [RoyalRoad](https://www.royalroad.com/)
//! - [FreeWebNovel](https://freewebnovel.com/)
//! - [LibRead](https://libread.com/)
//!
//! ## Cargo features
//!
//! Each available backend matches a [cargo `feature`](https://doc.rust-lang.org/cargo/reference/features.html) that can be enabled or
//! disabled.
//!
//! By default, only the *royalroad* and *freewebnovel* are enabled. *libread*
//! is disabled by default since (in my meager experience) it is simply a
//! different frontend for *freewebnovel*.
//!
//! if you want all features, including the default ones:
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! libwebnovel = {version="*", features = ["all"]}
//! ```
//!
//! ## Crate features / Task list
//!
//! - [ ] Find a way to handle something other than text content:
//!   - [ ] images
//!   - [ ] tables
//!   - [ ] chapter headers ?
//!   - [ ] chapter footers ?
//! - [ ] Add more backends:
//!   - [x] libread
//!   - [x] freewebnovel
//!   - [x] royalroad
//!   - [ ] scribblehub - May be complicated because of cloudflare
//!   - [ ] suggestions?
//! - [ ] implement an `async` version to get a better throughput. May be
//!   important for images?
//! - [x] ~create a binary using this lib to save webnovels to disk. It may also
//!   serve as a sample implementation?~ See [libwebnovel-storage](https://crates.io/crates/libwebnovel-storage)
//! - [x] implement a way to get an [`Ordering`][std::cmp::Ordering] between
//!   chapters. That enables us to detect collisions and still sort chapters
//!   that may have their indexes altered, such as in the case of removal in the
//!   source.
//! - [x] Add a way to detect potential collisions without requesting each
//!   individual chapter.
//! - [x] Add a way to get the chapter url & parent fiction url from a given
//!   chapter.
//! - [x] ~maybe find a way to parse a chapter index/number as to not overwrite
//!   local files when chapters are deleted on the backend~ -> done via
//!   [`Backends::get_ordering_function`].
//! - [x] add a way to get the cover image of the fiction, for epub generation.
//!
//! ## Legal
//!
//! Without explicit refutation in the header of any file in this repository,
//! all files in this repository are considered under the terms of the AGPL-3
//! license (of which a copy can be found in the LICENSE file at the root of
//! this repository) and bearing the mention "Copyright (c) 2024 paulollivier &
//! contributors".
//!
//! Basically, please do not use this code without crediting its writer(s) or
//! for a commercial project.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters, Setters};
use log::{debug, trace};
use scraper::{Html, Selector};
use thiserror::Error;

/// implementations of backends
pub mod backends;
pub use backends::{Backend, Backends};

pub(crate) mod utils;

/// A chapter of a webnovel
#[derive(Getters, Setters, CopyGetters, Default, Clone, PartialEq)]
pub struct Chapter {
    /// Index of this chapter in the grand scheme of things.
    #[getset(get = "pub", set = "pub")]
    index: usize,
    /// Title of this chapter, if any.
    #[getset(get = "pub", set)]
    title: Option<String>,
    /// Content of this chapter.
    #[getset(get = "pub")]
    content: String,
    /// Where can this chapter be found?
    #[getset(get = "pub", set)]
    chapter_url: String,
    /// Where can the fiction this chapter is from be found?
    #[getset(get = "pub", set)]
    fiction_url: String,
    /// date this chapter was published.
    #[getset(get = "pub", set)]
    published_at: Option<DateTime<Utc>>,
    /// Arbitrary metadata added by the backend.
    #[getset(get = "pub", set)]
    metadata: HashMap<String, String>,
}

impl Debug for Chapter {
    #[allow(unused_variables, dead_code)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct Chapter<'a> {
            index: &'a usize,
            title: &'a Option<String>,
            chapter_url: &'a String,
            fiction_url: &'a String,
            published_at: &'a Option<DateTime<Utc>>,
            metadata: &'a HashMap<String, String>,
        }
        let Self {
            index,
            title,
            content: _,
            chapter_url,
            fiction_url,
            published_at,
            metadata,
        } = self;
        Debug::fmt(
            &Chapter {
                index,
                title,
                chapter_url,
                fiction_url,
                published_at,
                metadata,
            },
            f,
        )
    }
}

impl Chapter {
    fn set_content(&mut self, s: impl Into<String>) {
        self.content = Html::parse_fragment(&s.into())
            .html()
            .strip_prefix("<html>")
            .unwrap()
            .strip_suffix("</html>")
            .unwrap()
            .trim()
            .to_string();
    }

    /// Add a key/value pair to the chapter's metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Returned when parsing a chapter fails.
#[derive(Debug, Error)]
pub struct ChapterParseError {
    message: String,
}

impl Display for ChapterParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ChapterParseError {
    /// creates a new ChapterParseError from a given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Attempts to parse a string into a Chapter.
///
/// ```rust
/// use std::str::FromStr;
///
/// use libwebnovel::Chapter;
/// let chapter_str = r#"
/// <!--
/// index: 1
/// chapter_url: https://read.freewebnovel.me/the-guide-to-conquering-earthlings/chapter-1
/// fiction_url: https://freewebnovel.com/the-guide-to-conquering-earthlings.html
/// published_at: not_found
/// metadata:
///   authors: Ye Fei Ran, 叶斐然
/// -->
/// <h1 class="mainTitle">Chapter 1: 01</h1>
/// <div class="content">
/// <p>this is some sample content, whatever man.</p>
/// </div>
/// "#;
/// let chapter = Chapter::from_str(chapter_str).unwrap();
/// assert_eq!(chapter.title(), &Some("Chapter 1: 01".to_string()));
/// assert_eq!(chapter.index(), &1);
/// assert_eq!(
///     chapter.chapter_url(),
///     "https://read.freewebnovel.me/the-guide-to-conquering-earthlings/chapter-1"
/// );
/// assert_eq!(
///     chapter.fiction_url(),
///     "https://freewebnovel.com/the-guide-to-conquering-earthlings.html"
/// );
/// assert!(chapter.published_at().is_none());
/// assert_eq!(
///     chapter.content(),
///     "<p>this is some sample content, whatever man.</p>"
/// );
/// assert_eq!(
///     chapter.metadata().get("authors"),
///     Some(&"Ye Fei Ran, 叶斐然".to_string())
/// );
/// ```
impl FromStr for Chapter {
    type Err = ChapterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chapter = Chapter::default();
        let mut chapter_data = HashMap::new();
        let mut metadata = HashMap::new();
        let mut in_metadata = false;
        let mut in_chapter_data = false;
        let mut in_content = false;
        let mut content = String::new();

        for line in s.lines() {
            trace!("line: {}", line);
            if line.starts_with("<!--") {
                in_chapter_data = true;
                debug!("found chapter data start");
                continue;
            } else if line.starts_with("-->") {
                in_chapter_data = false;
                debug!("found chapter data end");
                continue;
            }

            if in_chapter_data {
                if line.starts_with("metadata:") {
                    debug!("found metadata start");
                    in_metadata = true;
                    continue;
                }
                if !line.starts_with("  ") && in_metadata {
                    debug!("found metadata end");
                    in_metadata = false;
                }
                let parts: Vec<&str> = line.trim().splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    if in_metadata {
                        debug!("found metadata {}={}", key, value);
                        metadata.insert(key.to_string(), value.to_string());
                    } else {
                        debug!("found chapter_data {}={}", key, value);
                        chapter_data.insert(key.to_string(), value.to_string());
                    }
                }
            } else if let Some(title) = line.strip_prefix("<h1 class=\"mainTitle\">") {
                chapter.set_title(Some(title.trim_end_matches("</h1>").to_string()));
            } else if line.starts_with("<div class=\"content\">") {
                content.push_str("<div class=\"content\">");
                in_content = true;
            } else if in_content {
                content.push_str(&format!("{}\n", line));
            }
        }
        chapter.set_index(
            chapter_data
                .get("index")
                .and_then(|s| s.parse().ok())
                .ok_or(ChapterParseError::new(format!(
                    "Invalid chapter index: {:?}",
                    chapter_data.get("index")
                )))?,
        );
        chapter.set_chapter_url(
            chapter_data
                .get("chapter_url")
                .map(|s| s.to_string())
                .ok_or(ChapterParseError::new(format!(
                    "Invalid chapter url: {:?}",
                    chapter_data.get("chapter_url")
                )))?,
        );
        chapter.set_fiction_url(
            chapter_data
                .get("fiction_url")
                .map(|s| s.to_string())
                .ok_or(ChapterParseError::new(format!(
                    "Invalid fiction url: {:?}",
                    chapter_data.get("fiction_url")
                )))?,
        );
        chapter.set_published_at(chapter_data.get("published_at").and_then(|s| {
            if s == "not_found" {
                None
            } else {
                Some(DateTime::parse_from_rfc3339(s).ok()?.with_timezone(&Utc))
            }
        }));
        chapter.set_metadata(metadata);
        chapter.set_content(
            Html::parse_fragment(&content)
                .select(&Selector::parse("div.content").unwrap())
                .nth(0)
                .unwrap()
                .inner_html(),
        );
        Ok(chapter)
    }
}

/// Implement [`Display`] for [`Chapter`] (and consequentially, [`ToString`]).
impl Display for Chapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str("<!--\n");
        s.push_str(&format!("index: {}\n", self.index));
        s.push_str(&format!("chapter_url: {}\n", self.chapter_url));
        s.push_str(&format!("fiction_url: {}\n", self.fiction_url));
        s.push_str(&format!(
            "published_at: {}\n",
            if let Some(dt) = self.published_at {
                dt.to_rfc3339()
            } else {
                "not_found".to_string()
            }
        ));

        s.push_str("metadata:\n");
        for (key, value) in &self.metadata {
            s.push_str(&format!("  {}: {}\n", key, value));
        }
        s.push_str("-->\n");
        if let Some(title) = &self.title {
            s.push_str(&format!("<h1 class=\"mainTitle\">{}</h1>\n", title));
        }
        s.push_str(&format!(
            "<div class=\"content\">\n{}\n</div>",
            self.content
        ));
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use indoc::indoc;
    use test_log::test;

    use crate::Chapter;

    #[test]
    fn test_chapter_display() {
        let mut chapter = Chapter::default();
        chapter.set_title(Some("title".to_string()));
        chapter.set_chapter_url("https://chapter.url/".to_string());
        chapter.set_fiction_url("https://fiction.url".to_string());
        chapter.set_index(1);
        chapter.published_at = None;
        chapter
            .metadata
            .insert("authors".to_string(), "Ye Fei Ran, 叶斐然".to_string());
        chapter.set_content("<p>Test content</p>".to_string());
        let s = chapter.to_string();
        assert_eq!(
            s,
            indoc! {
                r#"<!--
                index: 1
                chapter_url: https://chapter.url/
                fiction_url: https://fiction.url
                published_at: not_found
                metadata:
                  authors: Ye Fei Ran, 叶斐然
                -->
                <h1 class="mainTitle">title</h1>
                <div class="content">
                <p>Test content</p>
                </div>"#
            }
        );
    }
    #[test]
    fn test_chapter_to_string_and_back() {
        let mut chapter = Chapter::default();
        chapter.set_title(Some("title".to_string()));
        chapter.set_chapter_url("https://chapter.url/".to_string());
        chapter.set_fiction_url("https://fiction.url".to_string());
        chapter.set_index(1);
        chapter.published_at = None;
        chapter
            .metadata
            .insert("authors".to_string(), "Ye Fei Ran, 叶斐然".to_string());
        chapter.set_content("<p>test content</p>".to_string());
        let s = chapter.to_string();
        let chapter_2 = Chapter::from_str(&s).unwrap();
        assert_eq!(chapter, chapter_2);
    }
}
