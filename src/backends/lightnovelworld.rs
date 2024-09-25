use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

use chrono::NaiveDateTime;
use regex::Regex;
use scraper::{Html, Selector};

use crate::backends::BackendError::ParseError;
use crate::backends::{BackendError, ChapterListElem};
use crate::utils::get;
use crate::{Backend, Chapter};

/// Backend for lightnovelworld.com
pub struct LightNovelWorld {
    url: String,
    main_page: Html,
    chapter_list_page: Html,
}

impl Default for LightNovelWorld {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            main_page: Html::new_document(),
            chapter_list_page: Html::new_document(),
        }
    }
}

#[allow(unused_variables, dead_code)]
impl Debug for LightNovelWorld {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct LightNovelWorld<'a> {
            url: &'a String,
        }
        let Self {
            url,
            main_page: _,
            chapter_list_page: _,
        } = self;
        Debug::fmt(&LightNovelWorld { url }, f)
    }
}

/// Implementation of [`Backend`] for [Light Novel World](https://www.lightnovelworld.com)
impl Backend for LightNovelWorld {
    fn get_backend_regexps() -> Vec<Regex> {
        vec![
            Regex::new(r#"https://www.lightnovelworld.com/novel/(?<novel_slug>[a-z0-9\-]+)/?"#)
                .unwrap(),
        ]
    }

    fn get_backend_name() -> &'static str {
        "lightnovelworld"
    }

    fn new(url: &str) -> Result<Self, BackendError> {
        let resp = get(url)?;
        if !resp.status().is_success() {
            return Err(BackendError::RequestFailed {
                message: format!("could not get fiction URL {url}"),
                status: resp.status(),
                content: resp.text()?,
            });
        }
        let main_page = Html::parse_document(&resp.text()?);
        let chapter_list_page = get(format!("{}/chapters", url))?;
        if !chapter_list_page.status().is_success() {
            return Err(BackendError::RequestFailed {
                message: format!("could not get chapter page, although we could get the main fiction page. Generated chapters url:  {url}"),
                status: chapter_list_page.status(),
                content: chapter_list_page.text()?,
            });
        }
        let chapter_list_page = Html::parse_document(&chapter_list_page.text()?);
        Ok(Self {
            url: url.to_string(),
            main_page,
            chapter_list_page,
        })
    }

    fn title(&self) -> Result<String, BackendError> {
        static TITLE_SELECTOR: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("h1.novel-title").unwrap());
        Ok(self
            .main_page
            .select(&TITLE_SELECTOR)
            .map(|sel| sel.inner_html())
            .next()
            .ok_or(BackendError::ParseError(format!(
                "Could not parse page to find title: {}",
                self.url
            )))?
            .trim_matches('\n')
            .to_string())
    }

    fn immutable_identifier(&self) -> Result<String, BackendError> {
        let mut split = self.url.rsplitn(3, '/');
        if let Some(s) = split.next() {
            if !s.is_empty() {
                // if we don't have a trailing slash
                return Ok(s.to_string());
            }
            if let Some(s) = split.next() {
                return Ok(s.to_string());
            }
        }
        Err(ParseError(
            "Could not find a proper identifier.".to_string(),
        ))
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn cover_url(&self) -> Result<String, BackendError> {
        static COVER_IMAGE_SELECTOR: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("html head meta[property=\"og:image\"]").unwrap());
        Ok(self
            .main_page
            .select(&COVER_IMAGE_SELECTOR)
            .next()
            .unwrap()
            .attr("content")
            .unwrap()
            .to_string())
    }

    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        static AUTHOR_SELECTOR: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("div.author a span").unwrap());
        // There can be only one author
        Ok(vec![self
            .main_page
            .select(&AUTHOR_SELECTOR)
            .next()
            .ok_or(BackendError::ParseError(
                "Failed to find authors in fiction page".to_string(),
            ))?
            .inner_html()
            .to_string()])
    }

    fn get_chapter_list(&self) -> Result<Vec<ChapterListElem>, BackendError> {
        const _CHAPTER_LIST_PAGE_COUNT: usize = 100;
        static CHAPTER_LIST_PAGE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
            Selector::parse("article#chapter-list-page section#chpagedlist ul.pagination li")
                .unwrap()
        });
        static CHAPTER_LIST_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
            Selector::parse(
                "article#chapter-list-page section#chpagedlist.container ul.chapter-list li",
            )
            .unwrap()
        });
        static CHAPTER_LIST_SELECTOR_CHAPTER_NO: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("a span.chapter-no").unwrap());
        static CHAPTER_LIST_SELECTOR_CHAPTER_TITLE: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("a").unwrap());

        let chapter_pages_count = self
            .chapter_list_page
            .select(&CHAPTER_LIST_PAGE_SELECTOR)
            .count()
            - 1; // "next" button
        let mut current_page = self.chapter_list_page.clone();
        let mut chapters = Vec::new();
        let mut i = 1usize;
        loop {
            let page_chapters: Vec<ChapterListElem> = current_page
                .select(&CHAPTER_LIST_SELECTOR)
                .map(|sel| {
                    let chapter_no: usize = sel
                        .select(&CHAPTER_LIST_SELECTOR_CHAPTER_NO)
                        .next()
                        .unwrap()
                        .inner_html()
                        .parse()
                        .unwrap();
                    let chapter_title = sel
                        .select(&CHAPTER_LIST_SELECTOR_CHAPTER_TITLE)
                        .next()
                        .unwrap()
                        .attr("title")
                        .unwrap();
                    (chapter_no, chapter_title.to_string())
                })
                .collect();
            chapters.extend(page_chapters);
            if i < chapter_pages_count {
                i += 1;
                current_page = Html::parse_document(
                    &get(format!("{}/chapters?page={}", self.url, i))?.text()?,
                );
            } else {
                break;
            }
        }
        Ok(chapters)
    }

    fn get_chapter(&self, chapter_number: usize) -> Result<Chapter, BackendError> {
        static CHAPTER_CONTENT_SELECTOR: LazyLock<Selector> =
            LazyLock::new(|| Selector::parse("div#chapter-container").unwrap());
        static CHAPTER_TITLE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
            Selector::parse("article#chapter-article div.titles h1 span.chapter-title").unwrap()
        });
        static CHAPTER_PUBLISHED_AT_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
            Selector::parse("article#chapter-article section.page-in.content-wrap div.titles meta[itemprop='datePublished']").unwrap()
        });
        let url = format!("{}/chapter-{}", self.url, chapter_number);
        let chapter_page = get(&url)?;
        let chapter_content = Html::parse_document(&chapter_page.text()?);
        let chapter_title = chapter_content
            .select(&CHAPTER_TITLE_SELECTOR)
            .next()
            .unwrap()
            .inner_html();
        // FIXME: remove ads (<p class="…"> instead of <p>)
        let chapter_paragraphs = chapter_content
            .select(&CHAPTER_CONTENT_SELECTOR)
            .next()
            .unwrap()
            .inner_html()
            .lines()
            .filter(|line| line.starts_with("<p>"))
            .collect::<Vec<&str>>()
            .join("\n");
        let published_at_str = chapter_content
            .select(&CHAPTER_PUBLISHED_AT_SELECTOR)
            .next()
            .unwrap()
            .attr("content")
            .unwrap();
        let published_at =
            NaiveDateTime::parse_from_str(published_at_str, "%Y-%m-%dT%H:%M:%S")?.and_utc();
        let mut chapter = Chapter::default();
        chapter.set_index(chapter_number);
        chapter.set_title(Some(chapter_title));
        chapter.set_chapter_url(url);
        chapter.set_fiction_url(self.url().clone());
        chapter.set_published_at(Some(published_at.to_utc()));
        chapter.set_content(chapter_paragraphs);
        Ok(chapter)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::NaiveDateTime;

    use super::*;
    use crate::Backend;

    const TEST_URL: &str = "https://www.lightnovelworld.com/novel/the-perfect-run-24071713";
    type TestBackend = LightNovelWorld;

    #[test]
    fn test_chapter_to_string_and_back() {
        let b = TestBackend::new(TEST_URL).unwrap();
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

    #[test]
    fn test_chapter_equality() {
        let b = TestBackend::new(TEST_URL).unwrap();
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

    #[test]
    fn test_ordering_function() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let mut chapters = vec![
            b.get_chapter(2).unwrap(),
            b.get_chapter(1).unwrap(),
            b.get_chapter(4).unwrap(),
            b.get_chapter(3).unwrap(),
        ];
        chapters.sort_by(TestBackend::get_ordering_function());
        assert_eq!(*chapters[0].index(), 1);
        assert_eq!(*chapters[1].index(), 2);
        assert_eq!(*chapters[2].index(), 3);
        assert_eq!(*chapters[3].index(), 4);
    }

    #[test]
    fn test_title() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let title = b.title().unwrap();
        assert_eq!(title, "The Perfect Run");
    }
    #[test]
    fn test_immutable_identifier() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let immutable_identifier = b.immutable_identifier().unwrap();
        assert_eq!(immutable_identifier, "the-perfect-run-24071713");
    }

    #[test]
    fn test_cover_url() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let cover_url = b.cover_url().unwrap();
        assert_eq!(
            cover_url,
            "https://static.lightnovelworld.com/bookcover/300x400/01261-the-perfect-run.jpg"
        )
    }

    #[test]
    fn test_get_author() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let authors = b.get_authors().unwrap();
        assert_eq!(authors, vec!["Void Herald"]);
    }

    #[test]
    fn test_get_chapter_list() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let chapter_list = b.get_chapter_list().unwrap();
        assert_eq!(chapter_list.len(), 130);
    }

    #[test]
    fn test_get_chapter() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let chapter = b.get_chapter(1).unwrap();
        assert_eq!(*chapter.index(), 1usize);
        assert_eq!(chapter.title(), &Some("Chapter 1: Quicksave".to_string()));
        assert_eq!(
            chapter.chapter_url(),
            &"https://www.lightnovelworld.com/novel/the-perfect-run-24071713/chapter-1".to_string()
        );
        assert_eq!(
            chapter.fiction_url(),
            &"https://www.lightnovelworld.com/novel/the-perfect-run-24071713".to_string()
        );
        assert_eq!(
            chapter.published_at(),
            &Some(
                NaiveDateTime::parse_from_str("2021-10-17T08:09:31", "%Y-%m-%dT%H:%M:%S")
                    .unwrap()
                    .and_utc()
            )
        );
        assert_eq!(chapter.metadata().len(), 0);
    }
    #[test]
    fn test_chapter_ads_removal() {
        let b = TestBackend::new(TEST_URL).unwrap();
        let chapter = b.get_chapter(1).unwrap();
        let regex = Regex::new(r#"<p class=".*">"#).unwrap();
        assert!(regex.captures(chapter.content()).is_none())
    }
}
