use std::convert::TryFrom;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use askama::Template;
use chrono::prelude::*;
use fallible_iterator::FallibleIterator;
use git2::Commit;
use semver::Version;
use simple_error::SimpleError;

use crate::bumping::{BumpType, LogEntry};

/// Contains all data needed to write the changelog
#[derive(Template)]
#[template(path = "CHANGELOG.md", escape = "none")]
pub struct ChangeLog<'a> {
    pub version: Version,
    pub date: NaiveDate,
    pub fixes: Vec<LogEntry<'a>>,
    pub features: Vec<LogEntry<'a>>,
    pub breaking: Vec<LogEntry<'a>>,
    pub other: Vec<LogEntry<'a>>,
}

impl Default for ChangeLog<'_> {
    fn default() -> Self {
        let today = Local::today();
        ChangeLog {
            version: Version::parse("0.0.0").unwrap(),
            date: NaiveDate::from_yo(today.year(), today.ordinal()),
            fixes: vec![],
            features: vec![],
            breaking: vec![],
            other: vec![],
        }
    }
}

impl ChangeLog<'_> {
    pub fn new<'a, I: FallibleIterator<Item=Commit<'a>, Error=SimpleError>>(commits: I) -> ChangeLog<'a> {
        let mut result = ChangeLog::<'a>::default();
        let _ = commits.for_each(|ref commit| {
            let msg = commit.message().unwrap_or_default();
            let bump_type = BumpType::parse_commit_msg(msg);
            match LogEntry::try_from(commit.clone()) {
                Ok(entry) => match bump_type {
                    BumpType::Patch => result.fixes.push(entry),
                    BumpType::Minor => result.features.push(entry),
                    BumpType::Major => result.breaking.push(entry),
                    BumpType::None => result.other.push(entry),
                },
                _ => () // FIXME add logging
            }
            Ok(())
        });
        result
    }
}

pub fn save(path_buf: &PathBuf, content: &[u8], overwrite: bool) -> Result<(), Box<dyn Error>> {
    let mut previous_file_content = Vec::new();

    if !overwrite && path_buf.exists() {
        OpenOptions::new()
            .read(true)
            .open(path_buf)?
            .read_to_end(&mut previous_file_content)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path_buf)?;

    file.write_all(content)?;
    file.write_all(previous_file_content.as_ref())?;

    Ok(())
}
