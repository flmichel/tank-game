FROM messense/rust-musl-cross:x86_64-musl as builder
ENV SQLX_OFFLINE=true
WORKDIR /server
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /server/target/x86_64-unknown-linux-musl/release/server /server
COPY configuration.yaml .
ENTRYPOINT ["/server"]
EXPOSE 3000
