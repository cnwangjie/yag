use anyhow::{bail, Result};
use log::debug;
use std::fmt;

use serde_derive::*;

use crate::structs::{PaginationResult, PullRequest};

#[derive(Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: usize,
    pub interval: usize,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum GetAccessTokenResponse {
    Ok(AccessToken),
    Error {
        error: String,
        error_description: String,
        interval: Option<u64>,
    },
}
#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GitHubResponse<T> {
    Ok(T),
    Error {
        error: Option<String>,
        message: String,
    },
}

impl<T> GitHubResponse<T>
where
    T: fmt::Debug,
{
    #[inline]
    pub fn map<R, F>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&T) -> Result<R>,
    {
        match self {
            GitHubResponse::Ok(data) => f(data),
            GitHubResponse::Error { error, message } => {
                debug!("found an error: {:#?}", self);
                bail!(message.to_string())
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchResult<T> {
    total_count: u64,
    incomplete_results: bool,
    items: Vec<T>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pull {
    id: u64,
    html_url: String,
    title: String,
    user: User,
    number: u64,
    base: Option<Ref>,
    head: Option<Ref>,
    updated_at: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    login: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ref {
    #[serde(rename = "ref")]
    name: String,
}

impl From<Pull> for PullRequest {
    fn from(pr: Pull) -> Self {
        Self {
            id: pr.number,
            title: pr.title,
            author: pr.user.login,
            base: pr.base.map(|r| r.name),
            head: pr.head.map(|r| r.name),
            updated_at: pr.updated_at,
            url: pr.html_url,
        }
    }
}

impl<T: Clone, U: From<T>> From<SearchResult<T>> for PaginationResult<U> {
    fn from(result: SearchResult<T>) -> Self {
        Self {
            total: result.total_count,
            result: result.items.iter().map(|i| U::from(i.to_owned())).collect(),
        }
    }
}
