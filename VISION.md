## What-Bump – Vision

### Goals

- Automatically detect the correct version bump
- Automatically generate a changelog
- Rely on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) as a standard to allow the points above
- Do not assume anything about the build process or release model of your software, except that your software under GIT
- Do not require any other external tool or service

### Scope

#### Things we will not support:
- Plugins for build systems

#### Things we may support in the future:
- Repositories other than GIT
- Exporting some functionality as a library
- Additional standards for commit messages

You can, of course, implement anything you want on top of this tool on your own and maintain it, but we are not going to include it if it's something specific to a single use-case. We prefer snippets and posts on StackOverflow for that specific scenarios.

### Technical Considerations

- **Supported Operating Systems**
We support MacOS and Linux. 
While we endevour to do our best and keep what-bump portable, Windows is by necessity less supported. We will however accept fixes to make it work on Windows (but we will not accept Windows-only features).

- **Rust Version**
We require *stable* rust 1.36 or later.

- **Backports**:
We will not make new features available on older versions of this tool. Only the latest will have all the features.
