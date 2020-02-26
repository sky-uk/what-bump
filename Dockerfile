FROM rust:1.36 as build

WORKDIR /usr/src/myapp
COPY . .

RUN cargo build --release && cp target/release/what-bump /

FROM scratch
COPY --from=build /what-bump /what-bump

CMD /what-bump
