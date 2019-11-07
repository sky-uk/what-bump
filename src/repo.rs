use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fmt;

use git2::{Commit, Repository};
use simple_error::SimpleError;
use fallible_iterator::FallibleIterator;

/// A git Repository whose messages respect the Conventional Commits specification
///
/// Can be created from the path of the git repo.
pub struct ConventionalRepo(Repository);

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
    pub fn commits_up_to<'a>(&'a self, revision: &str) -> Result<CommitIterator<'a>, Box<dyn Error>> {
        let result = CommitIterator {
            up_to: self.0.revparse_single(revision)?.peel_to_commit()?,
            current_commit: self.0.head()?.peel_to_commit()?,
        };
        Ok(result)
    }
}

pub struct CommitIterator<'a> {
    up_to: Commit<'a>,
    current_commit: Commit<'a>,
}

impl<'a> FallibleIterator for CommitIterator<'a> {
    type Item = Commit<'a>;
    type Error = SimpleError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let n_parents = self.current_commit.parent_count();
        if self.up_to.id() == self.current_commit.id() || n_parents == 0 {
            Ok(None)
        } else {
            if n_parents > 1 {
                // FIXME we need to handle multi-parent commits, at least in the case where there's an
                // unambiguous chain to the `up_to` revision
                eprintln!("Commit {} has more than one parent", self.current_commit.id());
            }
            let result = self.current_commit.clone();
            self.current_commit = self.current_commit.parent(0).unwrap_or(self.up_to.clone());
            Ok(Some(result))
        }
    }
}