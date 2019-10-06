use std::cmp::Ordering;
use std::error::Error;

use git2::Repository;

use crate::CommitType::{Feature, Fix, Other};

fn main() -> Result<(), Box<dyn Error>> {
    let repo = Repository::open("./")?;

    let mut commit = repo.head()?.peel_to_commit()?;
    let mut max_commit: ConventionalCommit = Default::default();

    loop {
        let msg = commit.message().unwrap_or("<no commit message>");
        let conv_comm = parse_commit_message(msg);
        println!("{}: {:?}", msg, conv_comm);
        if conv_comm > max_commit {
            max_commit = conv_comm;
        }
        commit = match commit.parent(0) {
            Ok(parent) => parent,
            Err(_) => break,
        }
    }

    println!("Max commit: {:?}", max_commit);
    Ok(())
}

fn parse_commit_message(original_msg: &str) -> ConventionalCommit {
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

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq)]
enum CommitType {
    Other, Fix, Feature
}

#[derive(Debug, Eq, PartialEq)]
struct ConventionalCommit {
    typ: CommitType,
    breaking: bool,
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
