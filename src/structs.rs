use std::fmt::Display;

use colored::*;

pub struct PaginationResult<T> {
    pub total: u64,
    pub result: Vec<T>,
}

impl<T> PaginationResult<T> {
    pub fn new(result: Vec<T>, total: u64) -> Self {
        PaginationResult {
            total: total,
            result: result,
        }
    }
}

impl<T> Display for PaginationResult<T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.result.iter() {
            write!(f, "{}", item)?;
        }
        writeln!(f, "  {} {}", "total:".purple(), self.total)?;
        Ok(())
    }
}

pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub base: String,
    pub head: String,
    pub author: String,
    pub updated_at: String,
    pub url: String,
}

impl Display for PullRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = format!("#{}", self.id).green().bold();
        let title = self.title.white();
        let author = format!("<{}>", self.author).blue().bold();
        let head = format!("[{}]", self.head).cyan();
        writeln!(f, "{:>6} {} {} {}", id, title, author, head)?;
        if f.alternate() {
            writeln!(f, "    {} {}", "link:".bold(), self.url)?;
        }
        Ok(())
    }
}
