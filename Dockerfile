FROM rust:1.36

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["what-bump"]
