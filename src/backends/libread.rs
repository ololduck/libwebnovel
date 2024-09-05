use regex::Regex;
use scraper::{Html, Selector};

use crate::backends::{freewebnovel, Backend, BackendError};
use crate::utils::get;
use crate::Chapter;

/// A backend using [libread](https://libread.com). Honestly i don't know why i bothered with it, since i'm under the impression that most chapters redirect to Freewebnovel
#[derive(Debug)]
pub struct LibRead {
    url: String,
    page: Html,
}
impl Default for LibRead {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            page: Html::new_document(),
        }
    }
}

/// ```rust
/// use libwebnovel::{Backend, Backends};
/// let backend =
///     Backends::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
///         .unwrap();
/// assert_eq!(
///     backend.title().unwrap(),
///     "The Guide to Conquering Earthlings"
/// );
/// ```
impl Backend for LibRead {
    /// Creates a new libread backend from the given URL
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.title().unwrap(),
    ///     "The Guide to Conquering Earthlings"
    /// );
    /// ```
    fn new(url: &str) -> Result<Self, BackendError> {
        let req = get(url)?;
        if !req.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "{}: {}",
                req.status(),
                req.text()?
            )));
        }
        Ok(Self {
            url: url.to_string(),
            page: Html::parse_document(&req.text()?),
        })
    }

    /// Title of the fiction. See [Libread::new()] for docs.
    fn title(&self) -> Result<String, BackendError> {
        freewebnovel::title(&self.page)
    }

    /// Returns the URL of the fiction
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.url(),
    ///     "https://libread.com/libread/the-guide-to-conquering-earthlings-33024"
    /// );
    /// ```
    fn url(&self) -> String {
        self.url.clone()
    }

    /// returns the authors of the fiction, if any
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.get_authors().unwrap(),
    ///     vec!["Ye Fei Ran".to_string(), "叶斐然".to_string()]
    /// );
    /// ```
    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        freewebnovel::authors(&self.page)
    }

    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(r"https?://libread\.com/libread/\w+").unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "libread"
    }

    /// returns a chapter
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.get_chapter(1).unwrap().title(),
    ///     &Some("Chapter 1: 01".to_string())
    /// );
    /// ```
    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        let chapter_list_selector = Selector::parse(freewebnovel::CHAPTER_LIST_SELECTOR).unwrap();
        let chapter_url = self
            .page
            .select(&chapter_list_selector)
            .map(|select| select.attr("href").unwrap())
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://libread.com{}", chapter_url);
        println!("{:?}", chapter_url);
        let mut chapter = freewebnovel::get_chapter(chapter_url)?;
        chapter.index = chapter_number;
        Ok(chapter)
    }

    /// Returns the total count of chapters
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// assert_eq!(backend.get_chapter_count().unwrap(), 60);
    /// ```
    fn get_chapter_count(&self) -> Result<u32, BackendError> {
        freewebnovel::chapter_count(&self.page)
    }
}
