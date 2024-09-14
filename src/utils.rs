use std::sync::LazyLock;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::IntoUrl;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| ClientBuilder::new().user_agent(USER_AGENT).build().unwrap());

/// Just a custom get that sets a correct User-Agent & follows redirects
pub(crate) fn get(url: impl IntoUrl) -> reqwest::Result<reqwest::blocking::Response> {
    HTTP_CLIENT.get(url).send()
}
