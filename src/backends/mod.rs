use std::cmp::Ordering;
use std::fmt::Debug;

use regex::Regex;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[cfg(feature = "freewebnovel")]
pub use crate::backends::freewebnovel::FreeWebNovel;
#[cfg(feature = "libread")]
pub use crate::backends::libread::LibRead;
#[cfg(feature = "royalroad")]
pub use crate::backends::royalroad::RoyalRoad;
use crate::utils::get;
use crate::Chapter;

#[cfg(feature = "libread")]
mod libread;
#[cfg(feature = "royalroad")]
mod royalroad;

#[cfg(feature = "freewebnovel")]
mod freewebnovel;

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
    /// Used when [`reqwest::Response::status()`] returns something else than
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
    UnknownChapter(usize),
    /// We have attempted something on a chapter which required some information
    /// that the chapter doesn't have. Most likely, this is something to report.
    #[error("{msg} on Chapter {chapter_url}:", chapter_url=chapter.chapter_url())]
    MissingChapterInformation {
        /// A message identifying why this error has been returned
        msg: String,
        /// The [`Chapter`] the issue originated from
        chapter: Box<Chapter>,
    },
}

type ChapterOrderingFn = Box<dyn Fn(&Chapter, &Chapter) -> Ordering>;
pub(crate) type ChapterListElem = (usize, String);
impl TryFrom<&Chapter> for ChapterListElem {
    type Error = BackendError;

    fn try_from(value: &Chapter) -> Result<Self, Self::Error> {
        Ok((
            value.index,
            value
                .title()
                .as_ref()
                .ok_or(BackendError::MissingChapterInformation {
                    msg: "Could not find a valid title".to_string(),
                    chapter: Box::new(value.clone()),
                })?
                .to_string(),
        ))
    }
}

/// Must be implemented by each backend.
///
/// ## How to implement a new backend ?
///
/// First, you need to implement the [`Backend`] trait for a struct capable of
/// handling your favorite novelreading website. Have a look at the
/// implementation of [`RoyalRoad`] (if the `royalroad` feature is active) for
/// an example.
///
/// Second, you need to add a feature in [Cargo.toml](/Cargo.toml) with your
/// backend name & what it requires.
///
/// Third, you need to add your new backend to the variants of [`Backends`].
/// Don't forget to tag it with `#[cfg(feature = "my_backend_feature_name")]`.
///
/// Fourth, you need to change all the methods of [`Backends`] to accept your
/// new variant. Don't forget to tag the new variant behaviour with
/// `#[cfg(feature = "my_backend_feature_name")]`.
pub trait Backend: Default + Debug
where
    Self: Sized,
{
    /// Gets the list of url regexps supported by this backend.
    fn get_backend_regexps() -> Vec<Regex>;
    /// An identifier for the backend. It is static and can be used for
    /// long-term storage.
    fn get_backend_name() -> &'static str;
    /// Returns a function enabling chapter ordering. This is important to
    /// ensure that chapters may still be correctly sorted when the source
    /// chapters have been removed.
    fn get_ordering_function() -> ChapterOrderingFn;
    /// Creates a new instance of itself
    fn new(url: &str) -> Result<Self, BackendError>;
    /// Returns the title of the fiction
    fn title(&self) -> Result<String, BackendError>;
    /// Returns _something_ that can be used to identify this novel, and won't
    /// change if (for instance) the title changes.
    fn immutable_identifier(&self) -> Result<String, BackendError>;
    /// Returns the url of the fiction
    fn url(&self) -> String;
    /// Returns the fictions' cover URL, if any
    fn cover_url(&self) -> Result<String, BackendError>;

    /// Returns a list of authors, if any
    fn get_authors(&self) -> Result<Vec<String>, BackendError>;

    /// Returns a vector of available chapters _without requesting the chapters
    /// themselves_. The goal is to be able to detect collisions between
    /// something stored locally and a distant source.
    ///
    /// All vector elements must be a tuple `(chapter_index: usize,
    /// chapter_title: String)`.
    fn get_chapter_list(&self) -> Result<Vec<ChapterListElem>, BackendError>;

    /// Returns a single chapter. The chapter number need to be _unique_, as
    /// some webnovel platforms allow truncating the chapter list.
    fn get_chapter(&self, chapter_number: usize) -> Result<Chapter, BackendError>;

    /// Must return the total chapter count. Default implementation calls
    /// [`self.get_chapter_list().len()`][Backend::get_chapter_list()].
    fn get_chapter_count(&self) -> Result<usize, BackendError> {
        Ok(self.get_chapter_list()?.len())
    }

    /// Returns all chapters for this fiction. The default implementation simply
    /// calls [`Self::get_chapter`] repeatedly
    fn get_chapters(&self) -> Result<Vec<Chapter>, BackendError> {
        let mut chapters = Vec::new();
        for i in 1..self.get_chapter_count()? {
            let chapter = self.get_chapter(i)?;
            chapters.push(chapter);
        }
        Ok(chapters)
    }

    /// Returns the fictions' cover as a byte array, if any.
    fn cover(&self) -> Result<Vec<u8>, BackendError> {
        let resp = get(self.cover_url()?)?;
        if !resp.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Could not download cover image: {}",
                resp.status()
            )));
        }
        let image_bytes = resp.bytes()?;
        Ok(image_bytes.to_vec())
    }
}

/// Enum listing all available backends. A new backend may be constructed using
/// [`Backends::new`].
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
    #[cfg(feature = "freewebnovel")]
    /// A FreeWebNovel backend
    FreeWebNovel(FreeWebNovel),
}

impl Backends {
    /// Returns the ordering function specific to the underlying backend.
    ///
    /// # Panics
    ///
    /// Panics when `self` is [`Backends::Dumb`].
    pub fn get_ordering_function(&self) -> ChapterOrderingFn {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => RoyalRoad::get_ordering_function(),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => LibRead::get_ordering_function(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(_) => FreeWebNovel::get_ordering_function(),
        }
    }

    /// Creates a new [`Backends`] variant from the given URL.
    pub(crate) fn new_from_url(&self, url: &str) -> Result<Backends, BackendError> {
        match self {
            Backends::Dumb => Ok(Self::Dumb),
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => Ok(Self::RoyalRoad(RoyalRoad::new(url)?)),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => Ok(Self::LibRead(LibRead::new(url)?)),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(_) => Ok(Self::FreeWebNovel(FreeWebNovel::new(url)?)),
        }
    }

    /// Returns the regexps used by the underlying backend. [`Backends::Dumb`]
    /// returns an empty [`Vec`].
    pub fn get_backend_regexps(&self) -> Vec<Regex> {
        match self {
            Backends::Dumb => Vec::new(),
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => RoyalRoad::get_backend_regexps(),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => LibRead::get_backend_regexps(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(_) => FreeWebNovel::get_backend_regexps(),
        }
    }

    /// Returns the underlying backend name.
    pub fn get_backend_name(&self) -> &'static str {
        match self {
            Backends::Dumb => "dummy",
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(_) => RoyalRoad::get_backend_name(),
            #[cfg(feature = "libread")]
            Backends::LibRead(_) => LibRead::get_backend_name(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(_) => FreeWebNovel::get_backend_name(),
        }
    }
}

/// All possible backend variants are contained within [`Backends`]. This
/// implementation tries to dispatch method calls to their appropriate
/// implementors.
///
/// ```rust
/// use libwebnovel::{Backend, Backends};
/// let backend =
///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
/// assert_eq!(backend.title().unwrap(), "Mother of Learning");
/// ```
impl Backend for Backends {
    /// Not implemented for [`Backends`]. Use
    /// [`Backends::get_backend_regexps(&self)`][a] instead.
    ///
    /// [a]: Backends#method.get_backend_regexps
    fn get_backend_regexps() -> Vec<Regex> {
        unimplemented!()
    }

    /// Not implemented for [`Backends`]. Use
    /// [`Backends::get_backend_name(&self)`][a] instead.
    ///
    /// [a]: Backends#method.get_backend_name
    fn get_backend_name() -> &'static str {
        unimplemented!()
    }

    /// Can't implement this function for backends without reference to `self`.
    /// use [`Backends::get_ordering_function(&self)`][a] instead.
    ///
    /// [a]: Backends#method.get_ordering_function
    fn get_ordering_function() -> ChapterOrderingFn {
        unimplemented!()
    }

    /// Builds a new backend for a given URL. Auto-detects the backend to use
    /// from the given URL, returning [`BackendError::NoMatchingBackendFound`]
    /// if none could be found.
    ///
    /// ```rust
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

    /// Returns the title of the webnovel. See [`Backends::new`] for an example.
    fn title(&self) -> Result<String, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.title(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.title(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.title(),
        }
    }

    fn immutable_identifier(&self) -> Result<String, BackendError> {
        match self {
            // implement this on the model of self.title() please
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.immutable_identifier(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.immutable_identifier(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.immutable_identifier(),
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
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.url(),
        }
    }

    fn cover_url(&self) -> Result<String, BackendError> {
        // Write this function, on the model of the other functions in Backends
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(backend) => backend.cover_url(),
            #[cfg(feature = "libread")]
            Backends::LibRead(backend) => backend.cover_url(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(backend) => backend.cover_url(),
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
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.get_authors(),
        }
    }

    fn get_chapter_list(&self) -> Result<Vec<ChapterListElem>, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_chapter_list(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_chapter_list(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.get_chapter_list(),
        }
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
    fn get_chapter(&self, chapter_number: usize) -> Result<Chapter, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_chapter(chapter_number),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_chapter(chapter_number),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.get_chapter(chapter_number),
        }
    }

    /// Returns the total count of the webnovel's chapters.
    ///
    /// # Example
    /// ```
    /// use libwebnovel::{Backend, Backends};
    /// let backend =
    ///     Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    /// assert_eq!(backend.get_chapter_count().unwrap(), 109);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when used on the [`Backends::Dumb`] backend.
    fn get_chapter_count(&self) -> Result<usize, BackendError> {
        match self {
            Backends::Dumb => {
                unimplemented!()
            }
            #[cfg(feature = "royalroad")]
            Backends::RoyalRoad(b) => b.get_chapter_count(),
            #[cfg(feature = "libread")]
            Backends::LibRead(b) => b.get_chapter_count(),
            #[cfg(feature = "freewebnovel")]
            Backends::FreeWebNovel(b) => b.get_chapter_count(),
        }
    }
}
