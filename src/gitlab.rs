use reqwest::{Client, RequestBuilder, Method, Url};
use reqwest::header::HeaderMap;
use anyhow::Result;
use super::*;

pub struct GitLabClient {
  host: String,
  token: String,
  client: reqwest::Client,
}

impl GitLabClient {
  fn new(host: &str, token: &str) -> Self {
    let mut headers = HeaderMap::new();
    headers.insert("Private-Token", token.parse().unwrap());

    let client = Client::builder()
      .default_headers(headers)
      .build()
      .unwrap();

    GitLabClient {
      host: host.to_string(),
      token: token.to_string(),
      client: client,
    }
  }

  fn call(&self, method: Method, uri: &str) -> RequestBuilder {
    let mut url = Url::parse(&format!("https://{}", self.host)).unwrap();
    url.set_path(uri);

    self.client.request(method, url)
  }
}

pub struct GitLabRepository {
  pub profile: Profile
}

impl Repository for GitLabRepository {
  fn list_pull_requests(&self) -> Result<()> {
    Ok(())
  }

  fn create_pull_request(&self) -> Result<()> {
    Ok(())
  }
}
