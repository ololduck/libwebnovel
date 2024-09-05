use chrono::DateTime;
use log::debug;
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};

use crate::backends::{Backend, BackendError};
use crate::Chapter;

const CHAPTER_TITLE_SELECTOR: &str = "table#chapters tbody tr.chapter-row td:first-child a";
const CHAPTER_CREATED_AT_SELECTOR: &str = "table#chapters tbody tr.chapter-row td:last-child time";
const FICTION_TITLE_SELECTOR: &str = "div.row.fic-header div.fic-title div.col h1.font-white";
const FICTION_AUTHORS_SELECTOR: &str = "div.row.fic-header div.fic-title div.col h4 span a";
const CHAPTER_PAGE_TITLE_SELECTOR: &str = "div.row.fic-header div.row div h1.font-white";
const CHAPTER_PAGE_CONTENT: &str =
    "div.page-container div.page-content-wrapper div.page-content div.container.chapter-page div div div.portlet-body div.chapter-inner.chapter-content";

/// A [Backend] implementation for [RoyalRoad](https://royalroad.com)
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

impl Backend for RoyalRoad {
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

    fn get_backend_regexps() -> Vec<Regex> {
        vec![Regex::new(
            r"https?://www.royalroad\.com/fiction/(?<fiction_id>\d+)/(?<fiction_title_slug>\w+)",
        )
        .unwrap()]
    }

    fn get_backend_name() -> &'static str {
        "royalroad"
    }

    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        let chapter_href_selector = Selector::parse(CHAPTER_TITLE_SELECTOR).unwrap();
        let chapter_date_selector = Selector::parse(CHAPTER_CREATED_AT_SELECTOR).unwrap();
        let chapter_url = self
            .fiction_page
            .select(&chapter_href_selector)
            .map(|select| select.attr("href").unwrap().to_string())
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_date = self
            .fiction_page
            .select(&chapter_date_selector)
            .map(|select| DateTime::parse_from_rfc3339(select.attr("datetime").unwrap()))
            .nth(chapter_number as usize - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        debug!("Attempting to get chapter {chapter_url}");
        let res = get(format!("https://www.royalroad.com{}", &chapter_url))?;
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
            .inner_html();
        Ok(Chapter {
            index: chapter_number,
            title: Some(chapter_title),
            content: chapter_content,
            published_at: Some(chapter_date?.to_utc()),
        })
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
