use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

use git2::Repository;
use semver::Version;
use structopt::StructOpt;

use crate::BumpType::{Major, Minor, Patch};
use crate::CommitType::{Feature, Fix, Other};

#[derive(Debug, StructOpt)]
#[structopt(name = "what-bump", about = "Automatically bump version based on conventional commits")]
struct Config {
    #[structopt(about = "Analyse commits up to this one")]
    up_to_revision: String,
    #[structopt(long, short)]
    from: Option<Version>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_args();
    let repo = Repository::open("./")?;

    let mut commit = repo.head()?.peel_to_commit()?;
    let mut max_commit: ConventionalCommit = Default::default();
    let up_to = repo.revparse_single(&config.up_to_revision)?.peel_to_commit()?;

    loop {
        let msg = commit.message().unwrap_or("<no commit message>");
        let conv_comm = ConventionalCommit::from(msg);
        if conv_comm > max_commit {
            max_commit = conv_comm;
        }

        if commit.id() == up_to.id() {
            break;
        }

        commit = match commit.parent(0) {
            Ok(parent) => parent,
            Err(_) => break,
        }
    }

    let bump_type: BumpType = max_commit.into();
    let output = config.from
        .map(|v| bump_type.bump(&v))
        .map(|v| v.to_string())
        .unwrap_or(format!("{}", bump_type))
    ;
    println!("{}", output);
    Ok(())
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq)]
enum CommitType {
    Other, Fix, Feature
}

impl Into<BumpType> for CommitType {
    fn into(self) -> BumpType {
        match self {
            Other => BumpType::None,
            Fix => Patch,
            Feature => Minor,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ConventionalCommit {
    typ: CommitType,
    breaking: bool,
}

impl From<&str> for ConventionalCommit {
    fn from(original_msg: &str) -> Self {
        let first_line = &original_msg[0..original_msg.find('\n').unwrap_or(original_msg.len())];
        let conventional_prefix = first_line[0..first_line.find(':').unwrap_or(first_line.len())].to_ascii_lowercase();
        let typ = if conventional_prefix.starts_with("fix") {
            Fix
        } else if conventional_prefix.starts_with("feat") {
            Feature
        } else {
            Other
        };

        let breaking = conventional_prefix.contains('!') || original_msg.contains("\nBREAKING CHANGE");

        ConventionalCommit{ typ, breaking }
    }
}

impl PartialOrd for ConventionalCommit {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            if self.breaking != other.breaking {
                if self.breaking {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                self.typ.cmp(&other.typ)
            }
        )
    }
}

impl Default for ConventionalCommit {
    fn default() -> Self {
        ConventionalCommit { typ: Other, breaking: false }
    }
}

impl Into<BumpType> for ConventionalCommit {
    fn into(self) -> BumpType {
        if self.breaking {
            Major
        } else {
            self.typ.into()
        }
    }
}

#[derive(Debug)]
enum BumpType {
    None, Patch, Minor, Major
}

impl Display for BumpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl BumpType {
    fn bump(&self, v: &Version) -> Version {
        let mut res = v.clone();
        match self {
            Patch => res.increment_patch(),
            Minor => res.increment_minor(),
            Major => res.increment_major(),
            _ => (),
        }
        res
    }
}