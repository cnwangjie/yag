use colored::*;

pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub base: String,
    pub head: String,
    pub author: String,
    pub updated_at: String,
    pub url: String,
}

impl PullRequest {
    pub fn println(&self) {
        let id = format!("#{}", self.id).green().bold();
        let title = self.title.white();
        let author = format!("<{}>", self.author).blue().bold();
        let head = format!("[{}]", self.head).cyan();
        println!("{:>6} {} {} {}", id, title, author, head);
    }

    pub fn print_detail(&self) {
        self.println();
        println!("    {} {}", "link:".bold(), self.url);
    }
}
