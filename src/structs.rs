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

impl<T: Display> Display for PaginationResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.result.iter() {
            writeln!(f, "{}", item)?;
        }
        writeln!(f, "  {} {}", "total:".purple(), self.total)?;
        Ok(())
    }
}

impl<T> PaginationResult<T> {
    #[inline]
    pub fn map<R, F>(&self, f: F) -> PaginationResult<R>
    where
        F: FnMut(&T) -> R,
    {
        PaginationResult {
            total: self.total,
            result: self.result.iter().map(f).collect::<Vec<R>>(),
        }
    }
}

pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub base: Option<String>,
    pub head: Option<String>,
    pub author: String,
    pub updated_at: String,
    pub url: String,
}

impl Display for PullRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = format!("#{}", self.id).green().bold();
        let title = self.title.white();
        let author = format!("<{}>", self.author).blue().bold();
        let head = self
            .head
            .to_owned()
            .map(|head| format!("[{}]", head).cyan())
            .unwrap_or_default();

        write!(f, "{:>6} {} {} {}", id, title, author, head)?;
        if f.alternate() {
            write!(f, "\n    {} {}", "link:".bold(), self.url)?;
        }
        Ok(())
    }
}
