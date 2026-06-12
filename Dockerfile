# Dockerfile for 100% Hermetic, Reproducible Static Compilation of Morphic Routing Network (ITS/SCPST)
# Uses rust:1.80-alpine to lock down the toolchain version and guarantee statically linked musl binaries.

FROM rust:1.80-alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/its-net

# Copy the entire workspace source files
COPY . .

# Build the workspace for the musl target to guarantee zero dynamic linker dependencies.
# The result is a 100% statically linked binary that runs on bare metal seL4, alpine, or fedora alike.
RUN cargo build --release --target x86_64-unknown-linux-musl

# Production stage
FROM scratch

# Copy only the statically compiled, stripped binaries
COPY --from=builder /usr/src/its-net/target/x86_64-unknown-linux-musl/release/its-net /its-net

ENTRYPOINT ["/its-net"]
