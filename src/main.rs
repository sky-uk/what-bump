use std::error::Error;
use std::path::PathBuf;

use fallible_iterator::FallibleIterator;
use semver::Version;
use simple_error::SimpleError;
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
    /// Analyse commits up to this one (exclusive).
    ///
    /// This would normally be the commit corresponding to your previous release (it can be
    /// a tag, a commit id, or anything else that GIT can parse).
    #[structopt(required_unless = "bump")]
    up_to_revision: Option<String>,

    /// Old version of your software.
    #[structopt(long, short)]
    from: Option<Version>,

    /// Location of the GIT repo.
    #[structopt(long, short, default_value = "./")]
    path: repo::ConventionalRepo,

    /// Also generate a changelog, and write it to this file.
    #[structopt(long, short)]
    changelog: Option<PathBuf>,

    /// Perform the specified version bump (you must also specify `--from`).
    ///
    /// Use this option if you know both the previous version, and the type of bump you need
    /// to do. This will skip the analysis of commit messages, therefore you don't need to
    /// provide a commit id if you use this option.
    #[structopt(long, short)]
    bump: Option<BumpType>,

    /// Overwrite the changelog file instead of prepending to the existing one.
    #[structopt(long, short)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::from_args();

    match (config.bump, &config.from) {
        (Some(ref bump_type), Some(ref version)) => {
            println!("{}", version.clone().bump(bump_type));
            return Ok(());
        },
        (Some(_), None) => {
            return Err(Box::new(SimpleError::new("If you specify `--bump`, you must also specify `--from`, otherwise I don't know what version to bump.")));
        }
        _ => ()
    }

    let up_to_revision = config.up_to_revision.unwrap();
    let max_bump_type = config.path.commits_up_to(&up_to_revision)?
        .map(|commit| Ok(BumpType::from(commit.message().unwrap_or("<no commit message>"))))
        .max()?
        .unwrap_or_default();

    let new_version = config.from.map(|v| v.bump(&max_bump_type));
    let output = new_version.clone()
        .map(|v| v.to_string())
        .unwrap_or(max_bump_type.to_string());

    if let Some(cl_path) = config.changelog {
        use askama::Template;

        let mut changelog = ChangeLog::new(config.path.commits_up_to(&up_to_revision)?);
        if let Some(new_version) = new_version {
            changelog.version = new_version;
        }
        changelog::save(cl_path, changelog.render()?.as_ref(), config.overwrite)?;
    }
    println!("{}", output);
    Ok(())
}
