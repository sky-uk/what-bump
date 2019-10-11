# What Bump?

Read commit history and decide what kind of version bump is required.

## Build

With Cargo, you can create a statically linked executable for your platform:

    cargo build

With docker, you can create a docker image:

    docker run --rm -it -v $PWD:/home/rust/src ekidd/rust-musl-builder cargo build --release
    