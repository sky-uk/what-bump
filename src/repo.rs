use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fmt;

use git2::{Commit, Repository};

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
    pub fn commits_up_to<'a>(&'a self, revision: &str) -> Result<impl Iterator<Item=Commit<'a>>, Box<dyn Error>> {
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