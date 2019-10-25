FROM rustlang/rust:nightly as builder

ENV APP_HOME /app/

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y upx musl-tools

COPY . $APP_HOME
WORKDIR $APP_HOME
RUN ls && cargo build --target x86_64-unknown-linux-musl --release --locked && strip target/x86_64-unknown-linux-musl/release/proby && upx target/x86_64-unknown-linux-musl/release/proby

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/proby /app/

ENTRYPOINT ["/app/proby"]
