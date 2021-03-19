use std::fmt;

use anyhow::{bail, Result};
use log::debug;
use serde_derive::*;
use serde_json::Value;

use crate::structs::PullRequest;

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub username: String,
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
    web_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GitLabResponse<T> {
    Data(T),
    Error {
        error: Option<Vec<String>>,
        message: Option<Value>,
    },
}

impl<T> GitLabResponse<T> where T: fmt::Debug {
    #[inline]
    pub fn map<R, F>(&self, f: F) -> Result<R> where F: FnOnce(&T) -> Result<R> {
        match self {
            GitLabResponse::Data(data) => f(data),
            GitLabResponse::Error { error, message } => {
                debug!("found an error: {:#?}", self);

                let message = message
                    .clone()
                    .and_then(|message| {
                        match message {
                            Value::String(message) => Some(message),
                            Value::Array(messages) => {
                                let message = messages
                                    .iter()
                                    .filter_map(|i| i.as_str())
                                    .map(|i| i.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                Some(message)
                            },
                            _ => None,
                        }
                    })
                    .or_else(move || error.clone().map(|i| i.join("\n")))
                    .unwrap_or("unknown error".to_string());

                bail!(message)
            },
        }
    }
}

impl<T> fmt::Display for GitLabResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            GitLabResponse::Error {
                error,
                message,
            } => {
                let msg = message.as_ref().and_then(|m| Some(format!("{:#}", m)))
                    .or_else(|| error.as_ref().and_then(|err| Some(err.join("\n"))))
                    .unwrap_or("unknown error".to_string());
                write!(f, "{}", msg)
            },
            _ => write!(f, "[data]")
        }
    }
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
            url: mr.web_url,
        }
    }
}
