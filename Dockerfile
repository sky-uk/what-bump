FROM scratch

COPY target/x86_64-unknown-linux-musl/release/what-bump .

CMD ["what-bump"]

