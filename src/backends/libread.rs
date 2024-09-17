use std::fmt::{Debug, Formatter};

use regex::Regex;
use scraper::Html;

use crate::backends::{
    freewebnovel, Backend, BackendError, ChapterListElem, ChapterOrderingFn, FreeWebNovel,
};
use crate::utils::get;
use crate::Chapter;

/// A backend using [libread](https://libread.com). Honestly i don't know why i bothered with it, since i'm under the impression that most chapters redirect to [FreeWebNovel](https://freewebnovel.com).
pub struct LibRead {
    url: String,
    page: Html,
}

#[allow(unused_variables, dead_code)]
impl Debug for LibRead {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct LibRead<'a> {
            url: &'a String,
        }
        let Self { url, page: _ } = self;
        Debug::fmt(&LibRead { url }, f)
    }
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
    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(r"https?://libread\.com/libread/\w+").unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "libread"
    }

    /// returns a function capable of comparing two chapters
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// let mut chapters = vec![
    ///     backend.get_chapter(2).unwrap(),
    ///     backend.get_chapter(1).unwrap(),
    ///     backend.get_chapter(4).unwrap(),
    ///     backend.get_chapter(3).unwrap(),
    /// ];
    /// chapters.sort_by(LibRead::get_ordering_function());
    /// assert_eq!(chapters[0].title(), &Some("Chapter 1: 01".to_string()));
    /// assert_eq!(chapters[1].title(), &Some("Chapter 2: The 02".to_string()));
    /// assert_eq!(chapters[2].title(), &Some("Chapter 3: 03".to_string()));
    /// assert_eq!(chapters[3].title(), &Some("Chapter 4: 04".to_string()));
    /// ```
    fn get_ordering_function() -> ChapterOrderingFn {
        FreeWebNovel::get_ordering_function()
    }

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

    /// Title of the fiction. See [`LibRead::new`] for docs.
    fn title(&self) -> Result<String, BackendError> {
        freewebnovel::title(&self.page)
    }

    fn immutable_identifier(&self) -> Result<String, BackendError> {
        Ok(self.url.split('/').last().unwrap().to_string())
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

    /// Returns the cover URL of the fiction
    ///
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// let cover_url = backend.cover_url().unwrap();
    /// assert_eq!(
    ///     cover_url,
    ///     "https://libread.com/files/article/image/4/4420/4420s.jpg"
    /// );
    /// ```
    fn cover_url(&self) -> Result<String, BackendError> {
        freewebnovel::get_cover_url(&self.page)
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

    /// Returns the chapter list as available on the main fiction page
    /// ```rust
    /// use libwebnovel::backends::LibRead;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
    ///         .unwrap();
    /// let chapter_lists = backend.get_chapter_list().unwrap();
    /// let expected_tuples: &[(usize, &str)] = &[
    ///     (1, "Chapter 1: 01"),
    ///     (2, "Chapter 2: The 02"),
    ///     (3, "Chapter 3: 03"),
    /// ];
    /// for (expected_index, expected_title) in expected_tuples {
    ///     assert_eq!(chapter_lists[*expected_index - 1].0, *expected_index);
    ///     assert_eq!(&chapter_lists[*expected_index - 1].1, expected_title);
    /// }
    /// ```
    fn get_chapter_list(&self) -> Result<Vec<ChapterListElem>, BackendError> {
        freewebnovel::get_chapter_list(&self.page)
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
    fn get_chapter(&self, chapter_number: usize) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        let chapter_url = self
            .page
            .select(&freewebnovel::CHAPTER_LIST_SELECTOR)
            .map(|select| select.attr("href").unwrap())
            .nth(chapter_number - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://libread.com{}", chapter_url);
        println!("{:?}", chapter_url);
        let mut chapter = freewebnovel::get_chapter(chapter_url)?;
        chapter.index = chapter_number;
        chapter.fiction_url = self.url.clone();
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
    fn get_chapter_count(&self) -> Result<usize, BackendError> {
        freewebnovel::chapter_count(&self.page)
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use test_log::test;

    use crate::backends::LibRead;
    use crate::{Backend, Chapter};

    #[test]
    fn test_chapter_to_string_and_back() {
        let b =
            LibRead::new("https://libread.com/libread/the-guide-to-conquering-earthlings-33024")
                .unwrap();
        let chapter = b.get_chapter(1).unwrap();
        let s = chapter.to_string();
        let chapter2 = Chapter::from_str(&s).unwrap();
        assert_eq!(chapter, chapter2);
    }
}
