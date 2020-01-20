# What Bump?

[![CircleCI](https://circleci.com/gh/sky-uk/what-bump.svg?style=svg&circle-token=bcb5547b4ce67d86715e1ce8f2a5c45b4bedba7d)](https://circleci.com/gh/sky-uk/what-bump)

`what-bump` is a simple tool that reads the commit history of a git repository,
and uses commit messages to decide what kind of version bump is required.

`what-bump` assumes that commit messages are written according to the 
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
specification, and that your software uses [Semantic Versioning](https://semver.org).

Moreover, if you provide `what-bump` with the current version number of your software,
it will tell you what the next version number has to be.

## Rationale

Plenty of tools exist that can read Conventional Commit messages and manage software
releases: see for example the [list](https://www.conventionalcommits.org/en/v1.0.0/#tooling-for-conventional-commits)
in the conventional commit website, as well as [calcver](https://github.com/sanisoclem/calcver-cli).

However, most of those tools are deeply integrated with NPM, or somehow assume that you're
using NPM, try to manage the entire release process of your software, and are not trivial
to set up (especially if you're not using NPM!). 
The only tool I could find that does not integrate with NPM is `calcver`, but it is very
young, very undocumented, and probably more powerful and/or complex than I need a tool 
to be.

This tool, `what-bump`, sets out to be a simple, self-explanatory, zero-configuration
utility to do one and only one thing: determine your software's next version number
based on all the commit messages up to a previous revision. You need to specify what
the previous revision is (we assume you have it tagged and know enough `bash` magic to
do it) and what the current version is (ditto).

## Usage

Just type

    what-bump --help

To get all the explanation you need. 

Basically, assuming you tagged your previous version as `v1.0.2`, just type
  
    what-bump v1.0.2 --from 1.0.2
  
to get the next version number printed to standard output.

## Compliance

`what-bump` is a little bit more accepting than Conventional Commits specifies.

In particular, it only checks that the commit type **starts** with "fix" or "feat" 
(case insensitive), therefore it will also accept things like "feature", "fixed", or
"fixing". 

Also, any other commit type, or commits that don't comply with the spec,
will be ignored and won't contribute to a version bump. 

## Build

### With Cargo

`what-bump` is written in [Rust](https://www.rust-lang.org). You'll need at least version 
1.36 to build (that's what I used). Install rust following the instructions on the official
website. 

To  install from [crates.io](https://crates.io) just type

    cargo install what-bump

It depends on [git2-rs](https://github.com/alexcrichton/git2-rs), which requires
libgit2 to be installed on your system. It should already be available if you're using git.

To build from sources, clone the repository and build with:

    cargo build --release
    
Then you can install it locally with

    cargo install --path .

### With Docker

Alternatively, if you have docker, you can build `what-bump` using 
the Dockerfile provided:

    docker build . -t what-bump

    
