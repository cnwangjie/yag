use anyhow::Result;
use log::debug;
use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, Response, Url};
use serde_json::json;

use super::structs::{DeviceCode, GetAccessTokenResponse};

const GITHUB_API_ENDPOINT: &str = "https://api.github.com";

pub struct GitHubClient {
    client: Client,
}

impl GitHubClient {
    fn get_default_headers() -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/vnd.github.v3+json".parse()?);
        headers.insert(
            "User-Agent",
            format!("yag/{}", env!("CARGO_PKG_VERSION")).parse()?,
        );
        Ok(headers)
    }

    pub fn build_with_basic_auth(username: &str, token: &str) -> Result<Self> {
        let mut headers = Self::get_default_headers()?;
        let token = base64::encode(format!("{}:{}", username, token));
        headers.insert("Authorization", format!("Basic {}", token).parse()?);
        debug!("default headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client: client })
    }

    pub fn build_with_oauth_token(token: &str) -> Result<Self> {
        let mut headers = Self::get_default_headers()?;
        headers.insert("Authorization", format!("token {}", token).parse()?);
        debug!("default headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client: client })
    }

    pub fn call(&self, method: Method, uri: &str) -> RequestBuilder {
        let mut url = Url::parse(GITHUB_API_ENDPOINT).unwrap();
        url.set_path(uri);

        self.client.request(method, url)
    }

    pub async fn graphql(&self, query: &str, variables: serde_json::Value) -> Result<Response> {
        Ok(self
            .call(Method::POST, "/graphql")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "query": query,
                    "variables": variables,
                })
                .to_string(),
            )
            .send()
            .await?)
    }
}

pub struct GitHubAnonymousClient {
    client: Client,
}

const CLIENT_ID: &str = "57dcd53cb489239f4c7b";

impl GitHubAnonymousClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert("Accept", "application/vnd.github.v3+json".parse()?);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client: client })
    }

    pub async fn gen_device_code(&self) -> Result<DeviceCode> {
        let res = self
            .client
            .post("https://github.com/login/device/code")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "client_id": CLIENT_ID,
                    "scope": "repo",
                })
                .to_string(),
            )
            .send()
            .await?;

        let text = res.text().await?;
        debug!("gen_device_code res: {}", text);

        let device_code = serde_json::from_str::<DeviceCode>(&text)?;

        Ok(device_code)
    }

    pub async fn gen_access_token(&self, device_code: &str) -> Result<GetAccessTokenResponse> {
        let res = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "client_id": CLIENT_ID,
                    "device_code": device_code,
                    "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                })
                .to_string(),
            )
            .send()
            .await?;

        let text = res.text().await?;
        debug!("gen_access_token res: {}", text);

        let access_token = serde_json::from_str::<GetAccessTokenResponse>(&text)?;

        Ok(access_token)
    }
}
