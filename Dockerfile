FROM clux/muslrust
WORKDIR /

# create a new empty project
RUN USER=root cargo new --lib dummy-project
WORKDIR /dummy-project

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm src/*

# copy source tree
COPY ./src ./src
COPY ./templates ./templates
RUN cargo build --release

FROM scratch
COPY --from=0 /dummy-project/target/x86_64-unknown-linux-musl/release/what-bump .

ENTRYPOINT ["/what-bump"]
