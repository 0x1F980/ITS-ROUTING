# Hermetic static musl build for its-routing (prebuilds v2).
# Runtime: alpine for docker compose exec + health/smoke checks.

FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/its-routing
COPY . .

RUN cargo build --release -p its_routing --target x86_64-unknown-linux-musl

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

COPY --from=builder /usr/src/its-routing/target/x86_64-unknown-linux-musl/release/its-routing /usr/local/bin/its-routing

ENTRYPOINT ["its-routing"]
CMD ["--help"]
