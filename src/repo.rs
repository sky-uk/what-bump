use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fmt;

use fallible_iterator::FallibleIterator;
use git2::{Commit, ObjectType, Repository, Revwalk};
use simple_error::SimpleError;

use crate::error::ToSimpleError;

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
        let up_to = self.0.revparse_single(revision)?.peel_to_commit()?;
        let mut walker= self.0.revwalk()?;
        walker.push_head()?;
        walker.hide(up_to.id())?;

        let result = CommitIterator {
            repo: &self.0,
            walker
        };
        Ok(result)
    }
}

pub struct CommitIterator<'a> {
    repo: &'a Repository,
    walker: Revwalk<'a>,
}

impl ToSimpleError for git2::Error {
    fn to_simple_error(&self) -> SimpleError {
        SimpleError::new(format!("{:?}", self))
    }
}

impl<'a> FallibleIterator for CommitIterator<'a> {
    type Item = Commit<'a>;
    type Error = SimpleError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {

        match self.walker.next() {
            Some(Ok(oid)) => Ok(Some(
                self.repo.find_object(oid, Some(ObjectType::Commit))
                    .map_err(|e| e.to_simple_error())?
                    .peel_to_commit()
                    .map_err(|e| e.to_simple_error())?
            )),
            Some(Err(error)) => Err(error.to_simple_error()),
            None => Ok(None),
        }
    }
}