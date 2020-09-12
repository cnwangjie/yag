use crate::profile::load_profile;
use crate::profile::Profile;
use crate::repository::Repository;
use crate::structs::MergeRequest;
use crate::structs::Project;
use crate::utils::url_encode;
use anyhow::Result;
use async_trait::async_trait;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde_json::json;

pub struct GitLabClient {
    host: String,
    token: String,
    client: reqwest::Client,
}

impl GitLabClient {
    fn build(host: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Private-Token", token.parse()?);
        debug!("default headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build()?;

        Ok(GitLabClient {
            host: host.to_string(),
            token: token.to_string(),
            client: client,
        })
    }

    fn call(&self, method: Method, uri: &str) -> RequestBuilder {
        let mut url = Url::parse(&format!("https://{}", self.host)).unwrap();
        url.set_path(uri);

        self.client.request(method, url)
    }

    async fn get_project_id(&self, name: &str) -> Result<u64> {
        let res = self.call(
            Method::GET,
            &format!("/api/v4/projects/{}", url_encode(name)),
        ).send().await?;

        let data = res.error_for_status()?.json::<Project>().await?;

        Ok(data.id)
    }
}

pub struct GitLabRepository {
    profile: Profile,
    client: GitLabClient,
    project_id: u64,
}

impl GitLabRepository {
    pub async fn init(host: &str, remote_url: &GitUrl) -> Result<Self> {
        let profile = load_profile().await?;
        let token = profile
            .get_token_by_host(host)
            .expect(&format!("unknown remote host: {}", host));
        let client = GitLabClient::build(host, &token)?;
        let project_id = client.get_project_id(remote_url.fullname.as_ref()).await?;
        Ok(GitLabRepository {
            profile,
            client,
            project_id,
        })
    }
}

#[async_trait]
impl Repository for GitLabRepository {
    async fn list_pull_requests(&self) -> Result<()> {
        let data = self
            .client
            .call(
                Method::GET,
                &format!(
                    "/api/v4/projects/{}/merge_requests?state=opened",
                    self.project_id
                ),
            )
            .send()
            .await?
            .json::<Vec<MergeRequest>>()
            .await?;

        debug!("{:#?}", data);
        Ok(())
    }

    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<()> {
        let data = self
            .client
            .call(
                Method::POST,
                &format!("/api/v4/projects/{}/merge_requests", self.project_id),
            )
            .header("Content-Type", "application/json")
            .body(
                json!({
                  "source_branch": source_branch,
                  "target_branch": target_branch,
                  "title": title,
                })
                .to_string(),
            )
            .send()
            .await?
            .text()
            .await?;

        debug!("{:#?}", data);
        Ok(())
    }
}
