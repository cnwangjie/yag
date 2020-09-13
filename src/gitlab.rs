use crate::profile::load_profile;
use crate::profile::Profile;
use crate::repository::Repository;
use crate::structs::PullRequest;
use crate::utils::url_encode;
use anyhow::Result;
use async_trait::async_trait;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde_derive::*;
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    id: u64,
    name: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MergeRequest {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    state: String,
    created_at: String,
    updated_at: String,
    target_branch: String,
    source_branch: String,
    author: User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GitLabResponse<T> {
    Error {
        error: Option<Vec<String>>,
        message: Option<String>,
    },
    Data(T),
}

impl From<MergeRequest> for PullRequest {
    fn from(mr: MergeRequest) -> Self {
        Self {
            id: mr.iid,
            title: mr.title,
            author: mr.author.username,
            base: mr.target_branch,
            head: mr.source_branch,
            updated_at: mr.updated_at,
        }
    }
}

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
        let res = self
            .client
            .call(
                Method::GET,
                &format!("/api/v4/projects/{}/merge_requests", self.project_id),
            )
            .query(&[("state", "opened"), ("per_page", "100")])
            .query(&[("page", 1)])
            .send()
            .await?;

        debug!("{:#?}", res);

        let data: Vec<MergeRequest> = res.json::<Vec<MergeRequest>>().await?;

        debug!("{:#?}", data);

        data.iter()
            .for_each(|mr| PullRequest::from(mr.to_owned()).print());
        Ok(())
    }

    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<()> {
        debug!("source: {} target: {}", source_branch, target_branch);
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
            .json::<GitLabResponse<MergeRequest>>()
            .await?;

        debug!("{:#?}", data);
        Ok(())
    }
}
