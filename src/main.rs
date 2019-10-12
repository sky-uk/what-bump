use std::error::Error;

use semver::Version;
use structopt::StructOpt;

use crate::bumping::{Bump, BumpType};

mod bumping;
mod repo;

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
    path: repo::ConventionalRepo,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::from_args();

    let max_bump_type = config.path.commits_up_to(&config.up_to_revision)?
        .map(|commit| BumpType::from(commit.message().unwrap_or("<no commit message")))
        .max()
        .unwrap_or_default();

    let output = config.from
        .map(|v| v.bump(&max_bump_type).to_string())
        .unwrap_or(max_bump_type.to_string());

    println!("{}", output);
    Ok(())
}
