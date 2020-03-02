use std::convert::TryFrom;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use chrono::prelude::*;
use fallible_iterator::FallibleIterator;
use git2::Commit;
use semver::Version;
use simple_error::SimpleError;
use log::error;
use tera::{Tera, Context};

use crate::bumping::{BumpType, LogEntry};

pub static DEFAULT_MD: &str = include_str!("../templates/default.md");

/// Contains all data needed to write the changelog
pub struct ChangeLog {
    pub version: Version,
    pub date: NaiveDate,
    pub fixes: Vec<LogEntry>,
    pub features: Vec<LogEntry>,
    pub breaking: Vec<LogEntry>,
    pub other: Vec<LogEntry>,
}

impl Default for ChangeLog {
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

impl<'a> std::convert::From<&ChangeLog> for Context {
    fn from(c: &ChangeLog) -> Self {
        let mut ctx = Context::new();
        ctx.insert("version", &c.version.to_string());
        ctx.insert("date", &c.date.to_string());
        ctx.insert("fixes", &c.fixes);
        ctx.insert("features", &c.features);
        ctx.insert("breaking", &c.breaking);
        ctx.insert("other", &c.other);
        ctx
    }
}

impl ChangeLog {
    pub fn new<'a, I: FallibleIterator<Item=Commit<'a>, Error=SimpleError>>(commits: I) -> ChangeLog {
        let mut result = ChangeLog::default();
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
                Err(e) => error!("{}", e),
            }
            Ok(())
        });
        result
    }

    pub fn save(&self, path_buf: &PathBuf, overwrite: bool) -> Result<(), Box<dyn Error>> {
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

        let mut tera = Tera::default();
        tera.add_raw_template("default.md", DEFAULT_MD)?;
        let result = tera.render("default.md", &self.into())?;

        file.write_all(result.as_ref())?;
        file.write_all(previous_file_content.as_ref())?;

        Ok(())
    }
}
