use serde_derive::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: u64,
    name: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MergeRequest {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: String,
    state: String,
    created_at: String,
    updated_at: String,
    target_branch: String,
    source_branch: String,
    author: User,
}
