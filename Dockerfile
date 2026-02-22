FROM alpine:latest
RUN mkdir app
COPY target/x86_64-unknown-linux-musl/release/yqwork-rust-2025 app/
RUN addgroup -S rust && adduser -S -G rust rust && \
    chown -R rust:rust /app
WORKDIR app
USER rust
CMD ["./yqwork-rust-2025"]