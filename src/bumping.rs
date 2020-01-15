use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use git2::Commit;
use semver::Version;
use simple_error::SimpleError;
use log::debug;

/// Extension methods for `&str`, useful for handling conventional commits
pub trait FirstLine<'a> {
    /// The first line of a conventional commit
    fn first_line(&'a self) -> &'a str;

    /// The part before the colon ":" symbol, i.e. type and optional scope
    fn prefix(&self) -> String;

    /// Split a conventional commit message into type and description
    fn split_at_colon(&'a self) -> (String, &'a str);

    /// Find a substring between (parenthesis) and return it if present
    fn extract_scope(&self) -> Option<String>;
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

    fn extract_scope(&self) -> Option<String> {
        match (self.find("("), self.find(")")) {
            (Some(open), Some(closed)) if closed > open => Some(self[open+1..closed].into()),
            _ => None
        }
    }
}

/// A change-log entry
///
/// Can be created from a git commit
pub struct LogEntry<'a> {
    pub scope: Option<String>,
    pub description: String,
    pub commit: Commit<'a>,
}

impl<'a> TryFrom<Commit<'a>> for LogEntry<'a> {
    type Error = Box<dyn Error>;

    fn try_from(commit: Commit<'a>) -> Result<LogEntry<'a>, Box<dyn Error>> {
        let commit_msg = commit.message().ok_or(SimpleError::new("No commit message"))?;
        let first_line = commit_msg.first_line();
        let (prefix, description) = first_line.split_at_colon();
        let description = description.trim();
        if description.is_empty() {
            return Err(Box::new(SimpleError::new(format!("Empty commit message in {}", commit.id()))));
        }
        Ok(LogEntry {
            scope: prefix.as_str().extract_scope(),
            description: description.to_owned(),
            commit,
        })
    }
}

/// The different types of version bumps that one can do
///
/// Can be created from a commit message.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum BumpType {
    None,
    Patch,
    Minor,
    Major,
}

/// Extension method for Version, that adds the ability to perform a version bump according to BumpType
pub trait Bump {
    fn bump(self, bt: &BumpType) -> Self;
}

/// Case-insensitive parse method for `BumpType`'s enum values
impl FromStr for BumpType {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(BumpType::None),
            "patch" => Ok(BumpType::Patch),
            "minor" => Ok(BumpType::Minor),
            "major" => Ok(BumpType::Major),
            x => Err(SimpleError::new(format!("Not a bump specification: {}", x)))
        }
    }
}

pub const OTHER_TYPES: &[&str] = &[
    "build",
    "ci",
    "chore",
    "docs",
    "feat",
    "fix",
    "perf",
    "refactor",
    "revert",
    "style",
    "test"
];

pub trait ObserveParseError {
    fn on_error(&mut self, commit_msg: &str, first_line: &str);
}

impl ObserveParseError for () {
    fn on_error(&mut self, _commit_msg: &str, _first_line: &str) {
        // No-op
    }
}

impl BumpType {

    pub fn parse_commit_msg(commit_msg: &str) -> Self {
        BumpType::parse_commit_msg_with_errors(commit_msg, &mut ())
    }

    /// Parses a conventional commit message into a `BumpType`. If a non-conventional message
    /// is encountered, the `error_observer` is called.
    ///
    /// A NO-OP implementation of `ObserveParseError` is provided for `()`, if you want to ignore
    /// errors. (Non-conventional messages always generate `BumpType::None`).
    pub fn parse_commit_msg_with_errors(commit_msg: &str, error_observer: &mut dyn ObserveParseError) -> Self {
        let first_line = commit_msg.first_line();
        let conventional_prefix = first_line.prefix();
        let breaking = conventional_prefix.contains('!') || commit_msg.contains("\nBREAKING CHANGE");
        let result =
            if breaking {
                BumpType::Major
            } else if conventional_prefix.starts_with("fix") {
                BumpType::Patch
            } else if conventional_prefix.starts_with("feat") {
                BumpType::Minor
            } else {
                if !OTHER_TYPES.contains(&conventional_prefix.as_str()) {
                    error_observer.on_error(commit_msg, first_line);
                }
                BumpType::None
            };
        debug!(r#"parsed "{}" into {} bump"#, first_line, result);
        result
    }
}

#[cfg(test)]
mod test {
    use crate::bumping::BumpType;

    #[test]
    fn test_parse_commit_messages() {
        assert_eq!(BumpType::parse_commit_msg("feat: hello"), BumpType::Minor);
        assert_eq!(BumpType::parse_commit_msg("FEAT: hello"), BumpType::Minor);
        assert_eq!(BumpType::parse_commit_msg("feat!: hello"), BumpType::Major);

        assert_eq!(BumpType::parse_commit_msg("fix: hello"), BumpType::Patch);
        assert_eq!(BumpType::parse_commit_msg("Fix: hello"), BumpType::Patch);
        assert_eq!(BumpType::parse_commit_msg("fix!: hello"), BumpType::Major);

        assert_eq!(BumpType::parse_commit_msg("chore"), BumpType::None);
        assert_eq!(BumpType::parse_commit_msg("platypus: foo"), BumpType::None);
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