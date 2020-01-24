FROM rust:alpine as builder

ENV APP_HOME /usr/src/app/

RUN apk add --update upx make

COPY . $APP_HOME
WORKDIR $APP_HOME
RUN make build-linux

FROM scratch
COPY --from=builder /usr/src/app/target/release/proby /app/

ENTRYPOINT ["/app/proby"]
