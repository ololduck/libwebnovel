use regex::Regex;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[cfg(feature = "libread")]
use crate::backends::libread::LibRead;
#[cfg(feature = "royalroad")]
use crate::backends::royalroad::RoyalRoad;
use crate::Chapter;

#[cfg(feature = "libread")]
mod libread;
#[cfg(feature = "royalroad")]
mod royalroad;

/// An error that may be returned when the backend encounters an error
#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    /// No backend capable of handling the given url has been found
    #[error("No backend has been found capable of handling the url {0}.")]
    NoMatchingBackendFound(String),
    /// Error while trying to access the fiction page
    #[error("An error has been encountered while trying to access fiction page: {0}")]
    NetError(#[from] reqwest::Error),
    /// We got an HTTP404 on the given URL :p
    #[error("the given url could not be found")]
    UrlNotFound,
    /// Used when [reqwest::Response::status()] returns something else than
    /// success
    #[error("We could not access the fiction page: {0}")]
    RequestFailed(String),
    /// An error while parsing the fiction
    #[error("An error occured while parsing the fiction page: {0}")]
    ParseError(String),
    /// We could not parse a valid date
    #[error("An error occured while trying to make sense of a date: {0}")]
    DateParseError(#[from] chrono::format::ParseError),
    /// returned when we could not find the given index for a chapter
    #[error("Could not find chapter {0}")]
    UnknownChapter(u32),
}

/// Must be implemented by each backend.
///
/// ## How to implement a new backend ?
/// There are multiple way to do this, but basically, you only need to implement the methods defined by this trait.
pub trait Backend
where
    Self: Sized,
{
    /// Creates a new instance of itself
    fn new(url: &str) -> Result<Self, BackendError>;
    /// Returns the title of the fiction
    fn title(&self) -> Result<String, BackendError>;
    /// returns the url of the fiction
    fn url(&self) -> String;

    /// Returns a list of authors, if any
    fn get_authors(&self) -> Result<Vec<String>, BackendError>;

    /// Gets the list of url regexps supported by this backend.
    fn get_backend_regexps() -> Vec<Regex>;
    /// An identifier.
    fn get_backend_name() -> &'static str;
    /// Returns a single chapter.
    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError>;
    /// Must return the total chapter count
    fn get_chapter_count(&self) -> Result<u32, BackendError>;

    /// Returns all chapters for this fiction. The default implementation simply
    /// calls [fn.get_chapter] repeatedly
    fn get_chapters(&self) -> Result<Vec<Chapter>, BackendError> {
        let mut chapters = Vec::new();
        for i in 0..self.get_chapter_count()? {
            let chapter = self.get_chapter(i)?;
            chapters.push(chapter);
        }
        Ok(chapters)
    }
}

/// Enum listing all available backends. A new backend may be constructed using
/// [Backends::new(url)].
#[derive(EnumCount, EnumIter, Debug, Default)]
pub enum Backends {
    /// A dumb backend that should never be constructed, but is necessary for
    /// iteration (with [`strum::EnumIter`] & other features.
    #[default]
    Dumb,
    #[cfg(feature = "royalroad")]
    /// A RoyalRoad backend
    RoyalRoad(RoyalRoad),
    #[cfg(feature = "libread")]
    /// A LibRead backend
    LibRead(LibRead),
}

impl Backends {
    pub(crate) fn new_from_url(&self, url: &str) -> Result<Backends, BackendError> {
        match self {
            Backends::Dumb => Ok(Self::Dumb),
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => Ok(Self::RoyalRoad(RoyalRoad::new(url)?)),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => Ok(Self::LibRead(LibRead::new(url)?)),
        }
    }
}

impl Backends {
    pub(crate) fn get_backend_regexps(&self) -> Vec<Regex> {
        match self {
            Backends::Dumb => Vec::new(),
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => RoyalRoad::get_backend_regexps(),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => LibRead::get_backend_regexps(),
        }
    }
}

impl Backend for Backends {
    /// Builds a new backend for a given URL.
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(backend.title().unwrap(), "Mother of Learning");
    /// ```
    fn new(url: &str) -> Result<Self, BackendError> {
        for backend_variant in Backends::iter() {
            for regex in backend_variant.get_backend_regexps() {
                if regex.is_match(url) {
                    return backend_variant.new_from_url(url);
                }
            }
        }
        Err(BackendError::NoMatchingBackendFound(url.to_string()))
    }

    /// Returns the title of the webnovel. See [Backends::new()] for an example.
    fn title(&self) -> Result<String, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.title(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.title(),
        }
    }

    /// Returns the URL of the webnovel.
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(
    ///     backend.url(),
    ///     "https://www.royalroad.com/fiction/21220/mother-of-learning"
    /// );
    /// ```
    fn url(&self) -> String {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.url(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.url(),
        }
    }

    /// Returns the author(s) of the webnovel
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(
    ///     backend.get_authors().unwrap(),
    ///     vec!["nobody103".to_string()]
    /// );
    /// ```
    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_authors(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_authors(),
        }
    }

    /// Not implemented for [`Backends`]
    fn get_backend_regexps() -> Vec<Regex> {
        unimplemented!()
    }

    /// Not implemented for [`Backends`]
    fn get_backend_name() -> &'static str {
        unimplemented!()
    }

    /// Returns a chapter of the webnovel, given its chapter number
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// let chapter = backend.get_chapter(1).unwrap();
    /// assert_eq!(
    ///     chapter.title(),
    ///     &Some("1. Good Morning Brother".to_string())
    /// );
    /// assert_eq!(*chapter.index(), 1);
    /// ```
    fn get_chapter(&self, chapter_number: u32) -> Result<Chapter, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_chapter(chapter_number),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_chapter(chapter_number),
        }
    }

    /// Returns the total count of the webnovel's chapters.
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(backend.get_chapter_count().unwrap(), 109);
    /// ```
    fn get_chapter_count(&self) -> Result<u32, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_chapter_count(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_chapter_count(),
        }
    }
}
