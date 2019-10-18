use std::convert::TryFrom;

use askama::Template;
use chrono::prelude::*;
use fallible_iterator::FallibleIterator;
use git2::Commit;
use semver::Version;
use simple_error::SimpleError;

use crate::bumping::{BumpType, LogEntry};
use std::error::Error;

#[derive(Template)]
#[template(path="CHANGELOG.md", escape = "none")]
pub struct ChangeLog<'a> {
    pub version: Version,
    pub date: NaiveDate,
    pub fixes: Vec<LogEntry<'a>>,
    pub features: Vec<LogEntry<'a>>,
    pub breaking: Vec<LogEntry<'a>>,
}

impl Default for ChangeLog<'_> {
    fn default() -> Self {
        let today = Local::today();
        ChangeLog {
            version: Version::parse("0.0.0").unwrap(),
            date: NaiveDate::from_yo(today.year(), today.ordinal()),
            fixes: vec![],
            features: vec![],
            breaking: vec![]
        }
    }
}

impl ChangeLog<'_> {
    pub fn new<'a, I: FallibleIterator<Item=Commit<'a>, Error=SimpleError>>(commits: I) -> Result<ChangeLog<'a>, Box<dyn Error>> {
        let mut result = ChangeLog::<'a>::default();
        commits.for_each(|ref commit| {
            let msg = commit.message().unwrap_or_default();
            let bump_type = BumpType::from(msg);
            match LogEntry::try_from(commit.clone()) {
                Ok(entry) => match bump_type {
                    BumpType::Patch => result.fixes.push(entry),
                    BumpType::Minor => result.features.push(entry),
                    BumpType::Major => result.breaking.push(entry),
                    BumpType::None => (),
                },
                _ => () // FIXME add logging
            }
            Ok(())
        })?;
        Ok(result)
    }
}