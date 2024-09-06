use std::collections::HashMap;

use chrono::DateTime;
use log::debug;
use regex::Regex;
use scraper::{Html, Selector};

use crate::backends::{Backend, BackendError, ChapterOrderingFn};
use crate::utils::get;
use crate::Chapter;

const CHAPTER_TITLE_SELECTOR: &str = "table#chapters tbody tr.chapter-row td:first-child a";
const CHAPTER_CREATED_AT_SELECTOR: &str = "table#chapters tbody tr.chapter-row td:last-child time";
const FICTION_TITLE_SELECTOR: &str = "div.row.fic-header div.fic-title div.col h1.font-white";
const FICTION_AUTHORS_SELECTOR: &str = "div.row.fic-header div.fic-title div.col h4 span a";
const CHAPTER_PAGE_TITLE_SELECTOR: &str = "div.row.fic-header div.row div h1.font-white";
const CHAPTER_PAGE_CONTENT: &str =
    "div.page-container div.page-content-wrapper div.page-content div.container.chapter-page div div div.portlet-body div.chapter-inner.chapter-content";

/// A [`Backend`] implementation for [RoyalRoad](https://royalroad.com)
#[derive(Debug)]
pub struct RoyalRoad {
    url: String,
    fiction_page: Html,
}
impl Default for RoyalRoad {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            fiction_page: Html::new_document(),
        }
    }
}

/// Builds a new RoyalRoad backend for a given URL.
/// ```
/// use libwebnovel::backends::RoyalRoad;
/// use libwebnovel::Backend;
/// let backend =
///     RoyalRoad::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
/// assert_eq!(backend.title().unwrap(), "Mother of Learning");
/// ```
impl Backend for RoyalRoad {
    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(
            r"https?://www\.royalroad\.com/fiction/(?<fiction_id>\d+)/(?<fiction_title_slug>\w+)",
        )
        .unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "royalroad"
    }

    /// Returns a function capable of comparing two chapters
    ///
    /// ```rust
    /// use libwebnovel::backends::RoyalRoad;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     RoyalRoad::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// let mut chapters = vec![
    ///     backend.get_chapter(2).unwrap(),
    ///     backend.get_chapter(1).unwrap(),
    ///     backend.get_chapter(4).unwrap(),
    ///     backend.get_chapter(3).unwrap(),
    /// ];
    /// chapters.sort_by(RoyalRoad::get_ordering_function());
    /// assert_eq!(
    ///     chapters[0].title(),
    ///     &Some("1. Good Morning Brother".to_string())
    /// );
    /// assert_eq!(
    ///     chapters[1].title(),
    ///     &Some("2. Life’s Little Problems".to_string())
    /// );
    /// assert_eq!(
    ///     chapters[2].title(),
    ///     &Some("3. The Bitter Truth".to_string())
    /// );
    /// assert_eq!(chapters[3].title(), &Some("4. Stars Fell".to_string()));
    /// ```
    fn get_ordering_function() -> ChapterOrderingFn {
        Box::new(|c1: &Chapter, c2: &Chapter| c1.published_at().cmp(c2.published_at()))
    }

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
            fiction_page: Html::parse_document(&req.text()?),
        })
    }

    fn title(&self) -> Result<String, BackendError> {
        let selector = Selector::parse(FICTION_TITLE_SELECTOR).unwrap();
        let title = self
            .fiction_page
            .select(&selector)
            .map(|selection| selection.inner_html())
            .next();
        debug!("Got title: {:?}", title);
        if title.is_none() {
            return Err(BackendError::ParseError(format!(
                "Failed to get title from {}",
                self.url
            )));
        }
        Ok(title.unwrap())
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        let selector = Selector::parse(FICTION_AUTHORS_SELECTOR).unwrap();
        let authors = self
            .fiction_page
            .select(&selector)
            .map(|selection| selection.inner_html())
            .next();
        if authors.is_none() {
            return Err(BackendError::ParseError(format!(
                "Failed to get authors from {}",
                self.url,
            )));
        }
        Ok(vec![authors.unwrap()])
    }

    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        // Create the CSS selectors
        let chapter_href_selector = Selector::parse(CHAPTER_TITLE_SELECTOR).unwrap();
        let chapter_date_selector = Selector::parse(CHAPTER_CREATED_AT_SELECTOR).unwrap();
        // Get che chapter URL
        let chapter_url = self
            .fiction_page
            .select(&chapter_href_selector)
            .map(|select| select.attr("href").unwrap().to_string())
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        // Get the chapter publication date
        let chapter_date = self
            .fiction_page
            .select(&chapter_date_selector)
            .map(|select| DateTime::parse_from_rfc3339(select.attr("datetime").unwrap()))
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://www.royalroad.com{}", chapter_url);
        let chapter_url_regex = Regex::new(r"https?://www\.royalroad\.com/fiction/(?<fiction_id>\d+)/(?<fiction_title_slug>[\w-]+)/chapter/(?<chapter_id>\d+)/(?<chapter_title_slug>[\w-]+)").unwrap();
        let matches = chapter_url_regex.captures(&chapter_url).unwrap();
        let metadata = HashMap::from([
            (
                "chapter_id".to_string(),
                matches.name("chapter_id").unwrap().as_str().to_string(),
            ),
            (
                "fiction_id".to_string(),
                matches.name("fiction_id").unwrap().as_str().to_string(),
            ),
        ]);

        debug!("Attempting to get chapter {chapter_url}");
        let res = get(&chapter_url)?;
        if !res.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "failed to get chapter {} from {}: {}",
                chapter_number,
                &chapter_url,
                res.text()?
            )));
        }
        let chapter_page = Html::parse_document(&res.text()?);
        let chapter_title = chapter_page
            .select(&Selector::parse(CHAPTER_PAGE_TITLE_SELECTOR).unwrap())
            .next()
            .unwrap()
            .inner_html()
            .trim_matches(&['\n', ' '])
            .to_string();
        let chapter_content = chapter_page
            .select(&Selector::parse(CHAPTER_PAGE_CONTENT).unwrap())
            .next()
            .unwrap()
            .inner_html()
            .to_string();
        let mut chapter = Chapter::default();
        chapter.set_index(chapter_number);
        chapter.set_title(Some(chapter_title));
        chapter.set_chapter_url(chapter_url);
        chapter.set_fiction_url(self.url().clone());
        chapter.set_published_at(Some(chapter_date?.to_utc()));
        chapter.set_metadata(metadata);
        chapter.set_content(chapter_content);
        Ok(chapter)
    }

    fn get_chapter_count(&self) -> Result<u32, BackendError> {
        let chapter_href_selector = Selector::parse(CHAPTER_TITLE_SELECTOR).unwrap();
        let chapter_urls: Vec<String> = self
            .fiction_page
            .select(&chapter_href_selector)
            .map(|select| select.attr("href").unwrap().to_string())
            .collect();
        Ok(chapter_urls.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use scraper::Html;
    use test_log::test;

    use crate::backends::RoyalRoad;
    use crate::{Backend, Chapter};

    const TEST_URL: &str = "https://www.royalroad.com/fiction/21220/mother-of-learning";
    #[test]
    fn test_chapter_to_string_and_back() {
        let b = RoyalRoad::new(TEST_URL).unwrap();
        let chapter = b.get_chapter(1).unwrap();
        let s = chapter.to_string();
        let chapter2 = Chapter::from_str(&s).unwrap();
        assert_eq!(chapter.index, chapter2.index);
        assert_eq!(chapter.title, chapter2.title);
        assert_eq!(chapter.chapter_url, chapter2.chapter_url);
        assert_eq!(chapter.fiction_url, chapter2.fiction_url);
        assert_eq!(chapter.published_at, chapter2.published_at);
        assert_eq!(chapter.metadata, chapter2.metadata);
        assert_eq!(
            Html::parse_fragment(&chapter.content),
            Html::parse_fragment(&chapter2.content)
        );
    }
}
