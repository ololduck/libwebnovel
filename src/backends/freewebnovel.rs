use log::debug;
use regex::Regex;
use reqwest::IntoUrl;
use scraper::{Html, Selector};

use crate::backends::BackendError;
use crate::utils::get;
use crate::{Backend, Chapter};

const TITLE_SELECTOR: &str = "h1.tit";
const AUTHORS_SELECTOR: &str = "a.a1";
pub(crate) const CHAPTER_LIST_SELECTOR: &str = "div.m-newest2 ul#idData li a.con";
const CHAPTER_TITLE_SELECTOR: &str = "div.top span.chapter";
const CHAPTER_CONTENT_SELECTOR: &str = "div.txt div#article";

/// An implementation of backend for https://freewebnovel.com
#[derive(Debug)]
pub struct FreeWebNovel {
    url: String,
    page: Html,
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

    /// Title of the fiction. See [FreeWebNovel::new()] for docs.
    fn title(&self) -> Result<String, BackendError> {
        title(&self.page)
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

    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(r"https?://freewebnovel\.com/[\w-]+\.html").unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "freewebnovel"
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
    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        let chapter_list_selector = Selector::parse(CHAPTER_LIST_SELECTOR).unwrap();
        let chapter_url = self
            .page
            .select(&chapter_list_selector)
            .map(|select| select.attr("href").unwrap())
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://freewebnovel.com{}", chapter_url);
        println!("{:?}", chapter_url);
        let mut chapter = get_chapter(chapter_url)?;
        chapter.index = chapter_number;
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
    fn get_chapter_count(&self) -> Result<u32, BackendError> {
        chapter_count(&self.page)
    }
}

pub(crate) fn get_chapter(url: impl IntoUrl) -> Result<Chapter, BackendError> {
    let resp = get(url)?;
    let page = Html::parse_document(&resp.text()?);
    let title_selector = Selector::parse(CHAPTER_TITLE_SELECTOR).unwrap();
    let content_selector = Selector::parse(CHAPTER_CONTENT_SELECTOR).unwrap();
    let chapter_title = page.select(&title_selector).next().unwrap().inner_html();
    let chapter_content = page.select(&content_selector).next().unwrap().inner_html();
    Ok(Chapter {
        index: 0,
        title: Some(chapter_title),
        content: chapter_content,
        published_at: None,
    })
}
pub(crate) fn title(page: &Html) -> Result<String, BackendError> {
    let selector = Selector::parse(TITLE_SELECTOR).unwrap();
    let title = page.select(&selector).map(|sel| sel.inner_html()).next();
    debug!("title: {:?}", title);
    if title.is_none() {
        return Err(BackendError::ParseError(
            "Could not get a title".to_string(),
        ));
    }
    Ok(title.unwrap())
}

pub(crate) fn authors(page: &Html) -> Result<Vec<String>, BackendError> {
    let selector = Selector::parse(AUTHORS_SELECTOR).unwrap();
    let authors = page
        .select(&selector)
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

pub(crate) fn chapter_count(page: &Html) -> Result<u32, BackendError> {
    let chapter_list_selector = Selector::parse(CHAPTER_LIST_SELECTOR).unwrap();
    let chapter_links: Vec<String> = page
        .select(&chapter_list_selector)
        .map(|select| select.attr("href").unwrap().to_string())
        .collect();
    Ok(chapter_links.len() as u32)
}
