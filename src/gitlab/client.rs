use anyhow::Result;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Url};
use crate::utils::url_encode;
use super::structs::Project;

pub struct GitLabClient {
  host: String,
  client: reqwest::Client,
}

impl GitLabClient {
  pub fn build(host: &str, token: &str) -> Result<Self> {
      let mut headers = HeaderMap::new();
      headers.insert("Private-Token", token.parse()?);
      debug!("default headers: {:?}", headers);
      let client = Client::builder().default_headers(headers).build()?;

      Ok(GitLabClient {
          host: host.to_string(),
          client: client,
      })
  }

  pub fn call(&self, method: Method, uri: &str) -> RequestBuilder {
      let mut url = Url::parse(&format!("https://{}", self.host)).unwrap();
      url.set_path(uri);

      self.client.request(method, url)
  }

  pub async fn get_project_id(&self, name: &str) -> Result<u64> {
      let res = self
          .call(
              Method::GET,
              &format!("/api/v4/projects/{}", url_encode(name)),
          )
          .send()
          .await?;

      let data = res.error_for_status()?.json::<Project>().await?;

      Ok(data.id)
  }
}
