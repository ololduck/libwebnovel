use regex::Regex;

use crate::backends::{Backend, BackendError};
use crate::Chapter;

#[derive(Debug, Default)]
pub struct LibRead {
    url: String,
}

impl Backend for LibRead {
    fn new(_url: &str) -> Result<Self, BackendError> {
        todo!();
    }

    fn title(&self) -> Result<String, BackendError> {
        todo!()
    }

    fn url(&self) -> String {
        todo!()
    }

    fn get_authors(&self) -> Result<Vec<String>, BackendError> {
        todo!()
    }

    fn get_backend_regexps() -> Vec<Regex> {
        todo!()
    }

    fn get_backend_name() -> &'static str {
        todo!()
    }

    fn get_chapter(&self, _chapter_number: u32) -> Result<Chapter, BackendError> {
        todo!()
    }

    fn get_chapter_count(&self) -> Result<u32, BackendError> {
        todo!()
    }
}
