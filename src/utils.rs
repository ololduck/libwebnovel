use reqwest::blocking::ClientBuilder;
use reqwest::IntoUrl;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Just a custom get that sets a correct User-Agent & follows redirects
pub(crate) fn get(url: impl IntoUrl) -> reqwest::Result<reqwest::blocking::Response> {
    let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;
    client.get(url).send()
}
