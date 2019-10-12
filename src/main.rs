mod bumping;

use std::error::Error;
use std::path::PathBuf;

use git2::Repository;
use semver::Version;
use structopt::StructOpt;

use crate::bumping::{BumpType, Bump};

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
    path: PathBuf
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_args();
    let repo = Repository::open(config.path)?;

    let mut commit = repo.head()?.peel_to_commit()?;
    let mut max_commit = BumpType::default();
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
