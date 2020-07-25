use std::error::Error;
use std::path::PathBuf;

use fallible_iterator::FallibleIterator;
use semver::Version;
use simple_error::SimpleError;
use structopt::StructOpt;
use log::{warn, error};
use lazy_static::lazy_static;

use crate::bumping::{Bump, BumpType};
use crate::changelog::{ChangeLog, TemplateType};

mod bumping;
mod repo;
mod changelog;
mod error;

lazy_static! {
    static ref STRICT_HELP: String = format!(r#"Quit with an error if non-conventional commit messages are found.

Default behaviour is to simply print a warning. Conventional messages start with
one of the following types: {}."#, bumping::OTHER_TYPES.join(", "));

    static ref TEMPLATE_ID_HELP: String = format!(r#"Use one of the changelog templates provided by `what-bump` (cannot be used with --template)

The available template IDs are: {}."#, changelog::DEFAULT_TEMPLATES.iter().map(|e| *e.0).collect::<Vec<_>>().join(", "));
}

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

    /// New version of your software (for changelog)
    ///
    /// Manually specify the new version of your software. Useful if you're just generating
    /// the changelog.
    #[structopt(long, short, requires = "changelog")]
    new_version: Option<String>,

    /// Location of the GIT repo.
    #[structopt(long, short, default_value = "./")]
    path: repo::ConventionalRepo,

    /// Also generate a changelog, and write it to this file.
    ///
    /// You can put placeholders between double-brackets (e.g. `{{version}}` or `{{date}}`),
    /// which will be substituted with their actual values.
    #[structopt(long, short)]
    changelog: Option<String>,

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

    #[structopt(long, help = &STRICT_HELP)]
    strict: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(long, short, parse(from_occurrences))]
    verbose: usize,

    /// Specify a custom template file for the changelog (cannot be used with --template-id)
    #[structopt(long, short)]
    template: Option<PathBuf>,

    #[structopt(long, help = &TEMPLATE_ID_HELP, default_value = "default.md")]
    template_id: String,
}

struct ParseError {
    first_line: String
}

impl bumping::ObserveParseError for Vec<ParseError> {
    fn on_error(&mut self, _commit_msg: &str, first_line: &str) {
        self.push(ParseError{ first_line: first_line.to_owned() });
    }
}

fn main() {
    let config: Config = Config::from_args();

    if let Err(e) = what_bump(config) {
        error!("{}", e);
        print_error_cause(e.as_ref());
        std::process::exit(1);
    }
}

fn print_error_cause(error: &dyn Error) {
    if let Some(error) = error.source() {
        error!("  caused by: {}", error);
        print_error_cause(error);
    }
}

fn what_bump(config: Config) -> Result<(), Box<dyn Error>> {
    stderrlog::new().module(module_path!()).verbosity(config.verbose + 2).init()?;

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

    let mut errors_found: Vec<ParseError> = Vec::new();
    let up_to_revision = config.up_to_revision.unwrap();
    let max_bump_type = config.path.commits_up_to(&up_to_revision)?
        .map(|commit| commit.message()
            .map(|m| BumpType::parse_commit_msg_with_errors(m, &mut errors_found) )
            .ok_or(SimpleError::new("No commit message"))
        )
        .max()?
        .unwrap_or_default();

    errors_found.iter().for_each(|e| warn!(r#"Not a conventional commit message: "{}""#, &e.first_line));
    if config.strict && !errors_found.is_empty() {
        return Err(Box::new(SimpleError::new("Some non-conventional commit messages were encountered while running in `strict` mode")));
    }

    let new_version = config.from.map(|v| v.bump(&max_bump_type));
    let output = new_version.clone()
        .map(|v| v.to_string())
        .unwrap_or(max_bump_type.to_string());

    if let Some(cl_path) = config.changelog {
        let mut changelog = ChangeLog::new(config.path.commits_up_to(&up_to_revision)?);
        if let Some(new_version) = config.new_version.or(new_version.map(|v| v.to_string())) {
            changelog.version = new_version;
        }
        changelog.save(&cl_path, config.overwrite, TemplateType::from_cli(config.template, config.template_id))?;
    }
    println!("{}", output);
    Ok(())
}
