mod bumping;

use std::error::Error;

use git2::{Repository, Commit};
use semver::Version;
use structopt::StructOpt;

use crate::bumping::{BumpType, Bump};
use std::fmt::{Debug, Formatter};
use std::fmt;

#[derive(Debug, StructOpt)]
#[structopt(name = "what-bump", about = r#"Detect version bump based on Conventional Commits

what-bump analyses your commit history, written according to the Conventional Commits specification (https://www.conventionalcommits.org/en/v1.0.0/), and outputs the type of version bump you need to do (one of Major, Minor, Patch, or None).

Optionally, if you specify the current version of your software, what-bump will print the bumped version (instead of the bump type).
"#)]
struct Config {
    #[structopt(about = "Analyse commits up to this one (exclusive)")]
    up_to_revision: String,
    #[structopt(long, short, help = "Current version of your software")]
    from: Option<Version>,
    #[structopt(long, short, default_value = "./", help = "Location of the GIT repo")]
    path: ConventionalRepo,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::from_args();

    let max_bump_type = config.path.commits_up_to(&config.up_to_revision)?
        .map(|commit| BumpType::from(commit.message().unwrap_or("<no commit message")))
        .max()
        .unwrap_or_default();

    let output = config.from
        .map(|v| v.bump(&max_bump_type).to_string())
        .unwrap_or(format!("{}", max_bump_type));

    println!("{}", output);
    Ok(())
}

struct ConventionalRepo(Repository);

impl Debug for ConventionalRepo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.path().fmt(f)
    }
}

impl std::str::FromStr for ConventionalRepo {
    type Err = git2::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Repository::open(s).map(ConventionalRepo)
    }
}

impl ConventionalRepo {
    pub fn commits_up_to(&self, revision: &str) -> Result<CommitIterator, Box<dyn Error>> {
        let result = CommitIterator {
            up_to: self.0.revparse_single(revision)?.peel_to_commit()?,
            current_commit: self.0.head()?.peel_to_commit()?
        };
        Ok(result)
    }
}

struct CommitIterator<'a> {
    up_to: Commit<'a>,
    current_commit: Commit<'a>,
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = Commit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.up_to.id() == self.current_commit.id() {
            None
        } else {
            let result = self.current_commit.clone();
            // FIXME should warn and terminate iteration if there's more than one parent
            self.current_commit = self.current_commit.parent(0).unwrap_or(self.up_to.clone());
            Some(result)
        }
    }
}