use semver::Version;
use std::fmt::{Display, Formatter};
use git2::Commit;
use std::convert::TryFrom;
use std::error::Error;
use simple_error::SimpleError;

trait FirstLine<'a> {
    fn first_line(&'a self) -> &'a str;
    fn prefix(&self) -> String;
    fn split_at_colon(&'a self) -> (String, &'a str);
}

impl FirstLine<'_> for &str {
    fn first_line(&self) -> &str {
        &self[0..self.find('\n').unwrap_or(self.len())]
    }

    fn prefix(&self) -> String {
        self[0..self.find(':').unwrap_or(self.len())].to_ascii_lowercase()
    }

    fn split_at_colon(&self) -> (String, &str) {
        let prefix = self.prefix();
        let description = if prefix.len() < self.len() {
            &self[prefix.len() + 1..self.len()]
        } else {
            ""
        };
        (prefix, description)
    }
}

pub struct LogEntry<'a> {
    pub scope: Option<String>,
    pub description: String,
    pub commit: &'a Commit<'a>,
}

impl<'a> TryFrom<&'a Commit<'a>> for LogEntry<'a> {
    type Error = Box<dyn Error>;

    fn try_from(commit: &'a Commit<'a>) -> Result<LogEntry<'a>, Box<dyn Error>> {
        let commit_msg = commit.message().ok_or(SimpleError::new("No commit message"))?;
        if BumpType::from(commit_msg) == BumpType::None {
            return Err(Box::new(SimpleError::new("Not a conventional commit")));
        }
        let first_line = commit_msg.first_line();
        let (_, description) = first_line.split_at_colon();
        let scope = None;
        Ok(LogEntry { scope, description: description.to_owned(), commit })
    }
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum BumpType {
    None,
    Patch,
    Minor,
    Major,
}

pub trait Bump {
    fn bump(self, bt: &BumpType) -> Self;
}

impl From<&str> for BumpType {
    fn from(commit_msg: &str) -> Self {
        let first_line = commit_msg.first_line();
        let conventional_prefix = first_line.prefix();
        let breaking = conventional_prefix.contains('!') || commit_msg.contains("\nBREAKING CHANGE");

        if breaking {
            BumpType::Major
        } else if conventional_prefix.starts_with("fix") {
            BumpType::Patch
        } else if conventional_prefix.starts_with("feat") {
            BumpType::Minor
        } else {
            BumpType::None
        }
    }
}

impl Default for BumpType {
    fn default() -> Self {
        BumpType::None
    }
}

impl Display for BumpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Bump for Version {
    fn bump(mut self, bt: &BumpType) -> Version {
        match bt {
            BumpType::Patch => self.increment_patch(),
            BumpType::Minor => self.increment_minor(),
            BumpType::Major => self.increment_major(),
            BumpType::None => (),
        }
        self
    }
}