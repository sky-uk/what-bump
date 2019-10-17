use git2::Commit;

use askama::Template;
use semver::Version;
use chrono::prelude::*;
use crate::bumping::{BumpType, LogEntry};

#[derive(Template)]
#[template(path="CHANGELOG.md", escape = "none")]
pub struct ChangeLog<'a> {
    pub version: Version,
    pub date: Date<Local>,
    pub fixes: Vec<LogEntry<'a>>,
    pub features: Vec<LogEntry<'a>>,
    pub breaking: Vec<LogEntry<'a>>,
}

impl ChangeLog<'_> {
    pub fn new<'a>(commits: &mut dyn Iterator<Item=Commit>) -> ChangeLog<'a> {
        let result = ChangeLog {
            version: Version::parse("0.0.0").unwrap(),
            date: Local::today(),
            fixes: vec![],
            features: vec![],
            breaking: vec![]
        };
        for commit in commits {
            let msg = commit.message().unwrap_or_default();
            let bump_type = BumpType::from(msg);
            if bump_type == BumpType::None { continue; }
        }
        result
    }
}