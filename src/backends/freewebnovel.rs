use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

use log::debug;
use regex::Regex;
use reqwest::IntoUrl;
use scraper::{Html, Selector};

use crate::backends::{BackendError, ChapterListElem, ChapterOrderingFn};
use crate::utils::get;
use crate::{Backend, Chapter};

pub(crate) static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1.tit").unwrap());
pub(crate) static AUTHORS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a.a1").unwrap());
pub(crate) static CHAPTER_LIST_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.m-newest2 ul#idData li a.con").unwrap());
pub(crate) static CHAPTER_TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.top span.chapter").unwrap());
pub(crate) static CHAPTER_CONTENT_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.txt div#article").unwrap());
pub(crate) static FICTION_COVER_IMAGE_URL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='og:image']").unwrap());

/// An implementation of backend for [FreeWebNovel](https://freewebnovel.com)
pub struct FreeWebNovel {
    url: String,
    page: Html,
}

#[allow(unused_variables, dead_code)]
impl Debug for FreeWebNovel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct FreeWebNovel<'a> {
            url: &'a String,
        }
        let Self { url, page: _ } = self;
        Debug::fmt(&FreeWebNovel { url }, f)
    }
}
impl Default for FreeWebNovel {
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
///     Backends::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html").unwrap();
/// assert_eq!(
///     backend.title().unwrap(),
///     "The Guide to Conquering Earthlings"
/// );
/// ```
impl Backend for FreeWebNovel {
    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(r"https?://freewebnovel\.com/[\w-]+\.html").unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "freewebnovel"
    }

    /// returns a function capable of comparing two chapters
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// let mut chapters = vec![
    ///     backend.get_chapter(2).unwrap(),
    ///     backend.get_chapter(1).unwrap(),
    ///     backend.get_chapter(4).unwrap(),
    ///     backend.get_chapter(3).unwrap(),
    /// ];
    /// chapters.sort_by(FreeWebNovel::get_ordering_function());
    /// assert_eq!(chapters[0].title(), &Some("Chapter 1: 01".to_string()));
    /// assert_eq!(chapters[1].title(), &Some("Chapter 2: The 02".to_string()));
    /// assert_eq!(chapters[2].title(), &Some("Chapter 3: 03".to_string()));
    /// assert_eq!(chapters[3].title(), &Some("Chapter 4: 04".to_string()));
    /// ```
    fn get_ordering_function() -> ChapterOrderingFn {
        fn parse_chapter_id(chapter_title: &str) -> Option<u32> {
            let re = Regex::new(r"Chapter (\d+)").unwrap();
            re.captures(chapter_title)
                .and_then(|caps| caps.get(1))
                .and_then(|cap| cap.as_str().parse::<u32>().ok())
        }

        Box::new(|c1: &Chapter, c2: &Chapter| {
            // parse the chapter title & extract the chapter number
            let chapter_number_1 = c1
                .title()
                .clone()
                .and_then(|title| parse_chapter_id(title.as_str()));

            let chapter_number_2 = c2
                .title()
                .clone()
                .and_then(|title| parse_chapter_id(title.as_str()));

            chapter_number_1.cmp(&chapter_number_2)
        })
    }

    /// Creates a new FreeWebNovel backend from the given URL
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.title().unwrap(),
    ///     "The Guide to Conquering Earthlings"
    /// );
    /// ```
    fn new(url: &str) -> Result<Self, BackendError> {
        let req = get(url)?;
        if !req.status().is_success() {
            return Err(BackendError::RequestFailed {
                message: format!("Could not fetch url {url}"),
                status: req.status(),
                content: req.text()?,
            });
        }
        Ok(Self {
            url: url.to_string(),
            page: Html::parse_document(&req.text()?),
        })
    }

    /// Title of the fiction. See [`FreeWebNovel::new`] for usage.
    fn title(&self) -> Result<String, BackendError> {
        title(&self.page)
    }

    fn immutable_identifier(&self) -> Result<String, BackendError> {
        Ok(self
            .url
            .split('/')
            .last()
            .unwrap()
            .to_string()
            .strip_suffix(".html")
            .unwrap()
            .to_string())
    }

    /// Returns the URL of the fiction
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.url(),
    ///     "https://freewebnovel.com/the-guide-to-conquering-earthlings.html"
    /// );
    /// ```
    fn url(&self) -> String {
        self.url.clone()
    }

    /// Returns the cover URL of the fiction
    ///
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// let cover_url = backend.cover_url().unwrap();
    /// assert_eq!(
    ///     cover_url,
    ///     "https://freewebnovel.com/files/article/image/4/4420/4420s.jpg"
    /// );
    /// ```
    fn cover_url(&self) -> Result<String, BackendError> {
        get_cover_url(&self.page)
    }

    /// returns the authors of the fiction, if any
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// assert_eq!(
    ///     backend.get_authors().unwrap(),
    ///     vec!["Ye Fei Ran".to_string(), "叶斐然".to_string()]
    /// );
    /// ```
    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        authors(&self.page)
    }

    /// Returns the chapter list as available on the main fiction page
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
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
        get_chapter_list(&self.page)
    }

    /// returns a chapter
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
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
            .select(&CHAPTER_LIST_SELECTOR)
            .map(|select| select.attr("href").unwrap())
            .nth(chapter_number - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://freewebnovel.com{}", chapter_url);
        let mut chapter = get_chapter(chapter_url)?;
        chapter.index = chapter_number;
        chapter.fiction_url = self.url.clone();
        Ok(chapter)
    }

    /// Returns the total count of chapters
    /// ```rust
    /// use libwebnovel::backends::FreeWebNovel;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     FreeWebNovel::new("https://freewebnovel.com/the-guide-to-conquering-earthlings.html")
    ///         .unwrap();
    /// assert_eq!(backend.get_chapter_count().unwrap(), 60);
    /// ```
    fn get_chapter_count(&self) -> Result<usize, BackendError> {
        chapter_count(&self.page)
    }
}

pub(crate) fn get_cover_url(page: &Html) -> Result<String, BackendError> {
    Ok(page
        .select(&FICTION_COVER_IMAGE_URL_SELECTOR)
        .next()
        .ok_or(BackendError::ParseError(
            "Could not find cover url".to_string(),
        ))?
        .attr("content")
        .ok_or(BackendError::ParseError(
            "Could not find cover url: missing \"content\" attribute.".to_string(),
        ))?
        .to_string())
}

pub(crate) fn get_chapter(url: impl IntoUrl) -> Result<Chapter, BackendError> {
    let url_str = url.into_url()?.to_string();
    let resp = get(&url_str)?;
    if !resp.status().is_success() {
        return Err(BackendError::RequestFailed {
            message: format!("Could not get chapter at URL {url_str}"),
            status: resp.status(),
            content: resp.text()?,
        });
    }
    let page = Html::parse_document(&resp.text()?);
    let chapter_title = page
        .select(&CHAPTER_TITLE_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    let chapter_content = page
        .select(&CHAPTER_CONTENT_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    let mut chapter = Chapter::default();
    chapter.set_title(Some(chapter_title));
    chapter.set_chapter_url(url_str);
    chapter.set_content(chapter_content);
    Ok(chapter)
}
pub(crate) fn title(page: &Html) -> Result<String, BackendError> {
    let title = page
        .select(&TITLE_SELECTOR)
        .map(|sel| sel.inner_html())
        .next();
    debug!("title: {:?}", title);
    if title.is_none() {
        return Err(BackendError::ParseError(
            "Could not get a title".to_string(),
        ));
    }
    Ok(title.unwrap())
}

pub(crate) fn authors(page: &Html) -> Result<Vec<String>, BackendError> {
    let authors = page
        .select(&AUTHORS_SELECTOR)
        .filter(|selection| {
            if let Some(href) = selection.attr("href") {
                return href.starts_with("/author/") || href.starts_with("/authors/");
            }
            false
        })
        .map(|a| a.inner_html())
        .collect();
    Ok(authors)
}

pub(crate) fn get_chapter_list(page: &Html) -> Result<Vec<ChapterListElem>, BackendError> {
    Ok(page
        .select(&CHAPTER_LIST_SELECTOR)
        .enumerate()
        .map(|(index, elem)| (index + 1, elem.attr("title").unwrap().to_string()))
        .collect())
}

pub(crate) fn chapter_count(page: &Html) -> Result<usize, BackendError> {
    let chapter_links: Vec<String> = page
        .select(&CHAPTER_LIST_SELECTOR)
        .map(|select| select.attr("href").unwrap().to_string())
        .collect();
    Ok(chapter_links.len())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use test_log::test;

    use crate::backends::FreeWebNovel;
    use crate::{Backend, Chapter};

    const TEST_URL: &str = "https://freewebnovel.com/the-guide-to-conquering-earthlings.html";

    #[test]
    fn test_chapter_to_string_and_back() {
        let b = FreeWebNovel::new(TEST_URL).unwrap();
        let chapter = b.get_chapter(1).unwrap();
        let s = chapter.to_string();
        let chapter2 = Chapter::from_str(&s).unwrap();
        assert_eq!(chapter, chapter2);
    }

    #[test]
    fn test_chapter_list_equality() {
        let b = FreeWebNovel::new(TEST_URL).unwrap();
        let chapters: Vec<Chapter> = (1..3)
            .map(|index| b.get_chapter(index).unwrap())
            .collect::<Vec<_>>();
        let expected = b.get_chapter_list().unwrap();
        for chapter in chapters {
            assert_eq!(
                chapter.title(),
                &Some(expected[chapter.index - 1].1.clone())
            )
        }
    }
}
