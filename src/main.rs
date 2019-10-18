use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use fallible_iterator::FallibleIterator;
use semver::Version;
use structopt::StructOpt;

use crate::bumping::{Bump, BumpType};
use crate::changelog::ChangeLog;

mod bumping;
mod repo;
mod changelog;

/// Detect version bump based on Conventional Commits
///
/// what-bump analyses your commit history, written according to the Conventional Commits specification
/// (https://www.conventionalcommits.org/en/v1.0.0/), and outputs the type of version bump you need to
/// do (one of Major, Minor, Patch, or None).
///
/// Optionally, if you specify the current version of your software, what-bump will print the bumped
/// version (instead of the bump type).
#[derive(Debug, StructOpt)]
#[structopt(name = "what-bump")]
struct Config {
    /// Analyse commits up to this one (exclusive)
    up_to_revision: String,

    /// Current version of your software
    #[structopt(long, short)]
    from: Option<Version>,

    /// Location of the GIT repo
    #[structopt(long, short, default_value = "./")]
    path: repo::ConventionalRepo,

    /// Also generate a changelog, and write it to this file
    #[structopt(long, short)]
    changelog: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::from_args();

    let max_bump_type = config.path.commits_up_to(&config.up_to_revision)?
        .map(|commit| Ok(BumpType::from(commit.message().unwrap_or("<no commit message>"))))
        .max()?
        .unwrap_or_default();

    let new_version = config.from.map(|v| v.bump(&max_bump_type));
    let output = new_version.clone()
        .map(|v| v.to_string())
        .unwrap_or(max_bump_type.to_string());

    if let Some(cl_path) = config.changelog {
        use askama::Template;

        let mut changelog = ChangeLog::new(config.path.commits_up_to(&config.up_to_revision)?)?;
        if let Some(new_version) = new_version {
            changelog.version = new_version;
        }
        let mut cl_file = File::create(cl_path)?;
        cl_file.write_all(changelog.render()?.as_ref())?;
    }
    println!("{}", output);
    Ok(())
}
