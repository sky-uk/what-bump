use std::convert::TryFrom;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::collections::HashMap;

use chrono::prelude::*;
use fallible_iterator::FallibleIterator;
use git2::Commit;
use semver::Version;
use simple_error::SimpleError;
use log::warn;
use tera::{Tera, Context};
use lazy_static::lazy_static;

use crate::bumping::{BumpType, LogEntry};

lazy_static! {
    pub static ref DEFAULT_TEMPLATES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("default.md", include_str!("../templates/default.md"));
        map.insert("default.html", include_str!("../templates/default.html"));
        map
    };
}

/// Identifies the template for changelog generation
pub enum TemplateType {
    /// a user-provided template
    File(PathBuf),
    /// one of what-bump's default templates
    Internal(String)
}

impl TemplateType {
    pub fn from_cli(template_file: Option<PathBuf>, template_id: String) -> Self {
        if let Some(path) = template_file {
            TemplateType::File(path)
        } else {
            TemplateType::Internal(template_id)
        }
    }
}

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
                Err(e) => warn!("{}", e),
            }
            Ok(())
        });
        result
    }

    pub fn save(&self, path_template: &String, overwrite: bool, template: TemplateType) -> Result<(), Box<dyn Error>> {
        let mut tera = Tera::default();
        let context = self.into();
        let mut previous_file_content = Vec::new();
        let path_buf = {
            tera.add_raw_template("<PATH>", &path_template)?;
            PathBuf::from(tera.render("<PATH>", &context)?)
        };

        if !overwrite && path_buf.exists() {
            OpenOptions::new()
                .read(true)
                .open(&path_buf)?
                .read_to_end(&mut previous_file_content)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path_buf)?;

        let template_name = match template {
            TemplateType::Internal(name) => {
                tera.add_raw_template(&name, DEFAULT_TEMPLATES[name.as_str()])?;
                name
            }
            TemplateType::File(path) => {
                tera.add_template_file(&path, None)?;
                path.to_string_lossy().to_string()
            },
        };

        let result = tera.render(&template_name, &context)?;

        file.write_all(&result.as_ref())?;
        file.write_all(previous_file_content.as_ref())?;

        Ok(())
    }
}
