use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::DateTime;
use log::debug;
use regex::Regex;
use scraper::{Html, Selector};

use crate::backends::{Backend, BackendError, ChapterListElem, ChapterOrderingFn};
use crate::utils::get;
use crate::Chapter;

/// Used to return the chapter's <a> in the fiction's chapter list
static CHAPTER_TITLE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table#chapters tbody tr.chapter-row td:first-child a").unwrap()
});
/// Used to return the date of the chapter in the fiction chapter's list
static CHAPTER_CREATED_AT_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table#chapters tbody tr.chapter-row td:last-child time").unwrap()
});
/// Used to return the authors of the fiction
static FICTION_AUTHORS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='books:author']").unwrap());
/// Used to return the chapter's title on the chapter page
static CHAPTER_PAGE_TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.row.fic-header div.row div h1.font-white").unwrap());
/// Used to return the chapter content on the chapter page
static CHAPTER_PAGE_CONTENT: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("div.page-container div.page-content-wrapper div.page-content div.container.chapter-page div div div.portlet-body div.chapter-inner.chapter-content").unwrap()
});
/// Used to get the fiction's title on the main fiction page
static FICTION_TITLE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("div.row.fic-header div.fic-title div.col h1.font-white").unwrap()
});

/// Used to return the fiction's cover image
static FICTION_IMAGE_URL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='og:image']").unwrap());

/// This is text added to RoyalRoad (RR) chapters when reading them outside of
/// RR's website (i guess). I think it is better to remove them since it
/// interrupts the flow of reading, and we know it's from RR, since we are
/// attempting to contact it directly, at the demand of the user. Maybe add a
/// disclaimer as a footing to remind the content is from RR and preserve the
/// spirit of these additions?
/// On other news, if find it nice from RR to put this, from an author
/// standpoint.
const ROYALROAD_ANTI_THEFT_TEXT: &[&str] = &[
    "This content has been misappropriated from Royal Road; report any instances of this story if found elsewhere.",
    "Find this and other great novels on the author's preferred platform. Support original creators!"
];

static ROYALROAD_ANTI_THEFT_REGEXPS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ROYALROAD_ANTI_THEFT_TEXT
        .iter()
        .map(|t| Regex::new(&format!(r#"<p( class=".*")?>{}</p>"#, t)).unwrap())
        .collect()
});

/// Used to identify a chapter URL
static ROYALROAD_CHAPTER_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://www\.royalroad\.com/fiction/(?<fiction_id>\d+)/(?<fiction_title_slug>[\w-]+)/chapter/(?<chapter_id>\d+)/(?<chapter_title_slug>[\w-]+)").unwrap()
});

/// Used to strip RR's weird paragraph CSS classes
static ROYALROAD_P_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<p class=".*">"#).unwrap());

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
            r"https?://www\.royalroad\.com/fiction/(?<fiction_id>\d+)/(?<fiction_title_slug>[\w\-]+)",
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
        let title = self
            .fiction_page
            .select(&FICTION_TITLE_SELECTOR)
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

    /// ```rust
    /// use libwebnovel::backends::RoyalRoad;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     RoyalRoad::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(
    ///     backend.immutable_identifier().unwrap(),
    ///     "mother-of-learning-21220"
    /// );
    /// ```
    fn immutable_identifier(&self) -> Result<String, BackendError> {
        let regex = &Self::get_backend_regexps()[0];
        let matches = regex.captures(&self.url);
        if let Some(matches) = matches {
            let fiction_id = matches.name("fiction_id").unwrap();
            let fiction_title = matches.name("fiction_title_slug").unwrap();
            Ok(format!(
                "{}-{}",
                fiction_title.as_str(),
                fiction_id.as_str()
            ))
        } else {
            Err(BackendError::ParseError("Unable to parse URL".to_string()))
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    /// Returns the cover URL of the fiction.
    ///
    /// ```rust
    /// use libwebnovel::backends::RoyalRoad;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     RoyalRoad::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// let cover_url = backend.cover_url().unwrap();
    /// assert_eq!(cover_url, "https://www.royalroadcdn.com/public/covers-full/21220-mother-of-learning.jpg?time=1637247458");
    /// ```
    fn cover_url(&self) -> Result<String, BackendError> {
        let img_url = self
            .fiction_page
            .select(&FICTION_IMAGE_URL_SELECTOR)
            .next()
            .ok_or(BackendError::ParseError(
                "Could not find fiction cover image url".to_string(),
            ))?
            .attr("content")
            .ok_or(BackendError::ParseError(
                "Could not find property \"content\" when searching for cover image".to_string(),
            ))?;
        Ok(img_url.to_string())
    }

    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        let authors : Result<Vec<String>, BackendError>=
            self.fiction_page
                .select(&FICTION_AUTHORS_SELECTOR)
                .map(|selection| selection.attr("content").ok_or_else(|| BackendError::ParseError("Failed to find 'content' attribute while looking at <meta property='books:author'>".to_string())).map(|s| s.to_string())).collect();

        let authors = authors.map_err(|e| {
            BackendError::ParseError(format!("Failed to get authors from {}: {}", self.url, e))
        })?;
        if authors.is_empty() {
            return Err(BackendError::ParseError(format!(
                "Failed to get authors from {}: Resulting author list is empty",
                self.url
            )));
        }
        Ok(authors)
    }

    /// Returns the chapter list as available on the main fiction page
    /// ```rust
    /// use libwebnovel::backends::RoyalRoad;
    /// use libwebnovel::Backend;
    /// let backend =
    ///     RoyalRoad::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// let chapter_lists = backend.get_chapter_list().unwrap();
    /// let expected_tuples: &[(usize, &str)] = &[
    ///     (1, "1. Good Morning Brother"),
    ///     (2, "2. Life’s Little Problems"),
    ///     (3, "3. The Bitter Truth"),
    /// ];
    /// for (expected_index, expected_title) in expected_tuples {
    ///     assert_eq!(chapter_lists[*expected_index - 1].0, *expected_index);
    ///     assert_eq!(&chapter_lists[*expected_index - 1].1, expected_title);
    /// }
    /// ```
    fn get_chapter_list(&self) -> Result<Vec<ChapterListElem>, BackendError> {
        let results = self
            .fiction_page
            .select(&CHAPTER_TITLE_SELECTOR)
            .enumerate()
            .map(|(index, elem)| {
                let title = elem.inner_html().trim_matches('\n').trim().to_string();
                (index + 1, title)
            })
            .collect();
        Ok(results)
    }

    fn get_chapter(&self, chapter_number: usize) -> Result<Chapter, BackendError> {
        if chapter_number == 0 {
            return Err(BackendError::UnknownChapter(chapter_number));
        }
        // Get che chapter URL
        let chapter_url = self
            .fiction_page
            .select(&CHAPTER_TITLE_SELECTOR)
            .map(|select| select.attr("href").unwrap().to_string())
            .nth(chapter_number - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        // Get the chapter publication date
        let chapter_date = self
            .fiction_page
            .select(&CHAPTER_CREATED_AT_SELECTOR)
            .map(|select| DateTime::parse_from_rfc3339(select.attr("datetime").unwrap()))
            .nth(chapter_number - 1)
            .ok_or(BackendError::UnknownChapter(chapter_number))?;
        let chapter_url = format!("https://www.royalroad.com{}", chapter_url);
        let matches = ROYALROAD_CHAPTER_URL_REGEX.captures(&chapter_url).unwrap();
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
        // A bit of text transformation to get rid of RR's anti-theft added text
        let mut txt = res.text()?;
        for regex in ROYALROAD_ANTI_THEFT_REGEXPS.iter() {
            txt = regex.replace(&txt, "").to_string();
        }

        // FIXME: don't use such a heavy-handed approach. Use Html parsing and not the
        //        brute regex method.
        txt = ROYALROAD_P_REGEX.replace(&txt, "<p>").to_string();

        let chapter_page = Html::parse_document(&txt);
        let chapter_title = chapter_page
            .select(&CHAPTER_PAGE_TITLE_SELECTOR)
            .next()
            .unwrap()
            .inner_html()
            .trim_matches(&['\n', ' '])
            .to_string();
        let chapter_content = chapter_page
            .select(&CHAPTER_PAGE_CONTENT)
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

    fn get_chapter_count(&self) -> Result<usize, BackendError> {
        let chapter_urls: Vec<String> = self
            .fiction_page
            .select(&CHAPTER_TITLE_SELECTOR)
            .map(|select| select.attr("href").unwrap().to_string())
            .collect();
        Ok(chapter_urls.len())
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

    #[test]
    fn test_chapter_equality() {
        let b = RoyalRoad::new(TEST_URL).unwrap();
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
