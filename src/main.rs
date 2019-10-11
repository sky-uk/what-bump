use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use git2::Repository;
use semver::Version;
use structopt::StructOpt;

use crate::BumpType::{Major, Minor, Patch};

#[derive(Debug, StructOpt)]
#[structopt(name = "what-bump", about = r#"Detect version bump based on Conventional Commits

what-bump analyses your commit history, written according to the Conventional Commits specification (https://www.conventionalcommits.org/en/v1.0.0/), and outputs the type of version bump you need to do (one of Major, Minor, Patch, or None).

Optionally, if you specify the current version of your software, what-bump will print the bumped version (instead of the bump type).
"#)]
struct Config {
    #[structopt(about = "Analyse commits up to this one")]
    up_to_revision: String,
    #[structopt(long, short, help = "Current version of your software")]
    from: Option<Version>,
    #[structopt(long, short, default_value = "./", help = "Location of the GIT repo")]
    path: PathBuf
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_args();
    let repo = Repository::open(config.path)?;

    let mut commit = repo.head()?.peel_to_commit()?;
    let mut max_commit = BumpType::None;
    let up_to = repo.revparse_single(&config.up_to_revision)?.peel_to_commit()?;

    loop {
        if commit.id() == up_to.id() {
            break;
        }

        let msg = commit.message().unwrap_or("<no commit message>");
        let conv_comm = BumpType::from(msg);
        if conv_comm > max_commit {
            max_commit = conv_comm;
        }

        commit = match commit.parent(0) {
            Ok(parent) => parent,
            Err(_) => break,
        }
    }

    let bump_type: BumpType = max_commit.into();
    let output = config.from
        .map(|v| v.bump(&bump_type))
        .map(|v| v.to_string())
        .unwrap_or(format!("{}", bump_type))
    ;
    println!("{}", output);
    Ok(())
}

impl From<&str> for BumpType {
    fn from(original_msg: &str) -> Self {
        let first_line = &original_msg[0..original_msg.find('\n').unwrap_or(original_msg.len())];
        let conventional_prefix = first_line[0..first_line.find(':').unwrap_or(first_line.len())].to_ascii_lowercase();

        let breaking = conventional_prefix.contains('!') || original_msg.contains("\nBREAKING CHANGE");

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

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq)]
enum BumpType {
    None, Patch, Minor, Major
}

impl Display for BumpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

trait Bump {
    fn bump(self, bt: &BumpType) -> Version;
}

impl Bump for Version {
    fn bump(mut self, bt: &BumpType) -> Version {
        match bt {
            Patch => self.increment_patch(),
            Minor => self.increment_minor(),
            Major => self.increment_major(),
            BumpType::None => (),
        }
        self
    }
}