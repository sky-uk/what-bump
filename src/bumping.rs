use semver::Version;
use std::fmt::{Display, Formatter};

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
        let first_line = &commit_msg[0..commit_msg.find('\n').unwrap_or(commit_msg.len())];
        let conventional_prefix = first_line[0..first_line.find(':').unwrap_or(first_line.len())].to_ascii_lowercase();

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