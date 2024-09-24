use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

use log::{error, warn};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::{IntoUrl, StatusCode};

use crate::backends::BackendError;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| ClientBuilder::new().user_agent(USER_AGENT).build().unwrap());

/// Just a custom get that sets a correct User-Agent & follows redirects
pub(crate) fn get(url: impl IntoUrl) -> Result<Response, BackendError> {
    let url = url.into_url()?;
    let mut fibonacci_iterator = FibonacciIterator::new();
    let _ = fibonacci_iterator.next(); // get rid of the first value, which is 0
    loop {
        // FIXME: dont use clone()
        let response = HTTP_CLIENT.get(url.clone()).send()?;
        if response.status().is_success() {
            return Ok(response);
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let wait_time = fibonacci_iterator.next().unwrap();
            if wait_time > 60 {
                error!("URL {url} replied we sent too many requests too many times.");
                return Err(BackendError::RequestFailed {message: format!("Could not fetch {url}. Backend said we sent too many requests, and we have exhausted our number of retries"), status: response.status(), content: response.text()?});
            }
            warn!("URL {url} replied we sent too many requests. Will wait for {wait_time}s before trying again.");
            sleep(Duration::from_secs(wait_time as u64));
            continue;
        }
    }
}

struct FibonacciIterator {
    next: usize,
    current: usize,
}

impl FibonacciIterator {
    pub(crate) fn new() -> Self {
        FibonacciIterator {
            next: 1,
            current: 0,
        }
    }
}

impl Iterator for FibonacciIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let previous = self.current;
        self.current = self.next;
        self.next += previous;
        Some(previous)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::FibonacciIterator;

    #[test]
    fn test_fibonacci() {
        let iter = FibonacciIterator::new();
        assert_eq!(
            iter.take(10).collect::<Vec<_>>(),
            vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
        );
    }
}
