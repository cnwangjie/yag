
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub base: String,
    pub head: String,
    pub author: String,
    pub updated_at: String,
}

impl PullRequest {
    pub fn print(&self) {
        println!("{:>6} {} <{}> {} -> {}", self.id, self.title, self.author, self.head, self.base);
    }
}
