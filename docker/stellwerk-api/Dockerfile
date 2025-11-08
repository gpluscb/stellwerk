# syntax=docker/dockerfile:1

FROM rustlang/rust:nightly-bookworm-slim AS rust-build
WORKDIR /app

RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml,readonly \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock,readonly \
    --mount=type=bind,source=rust-toolchain.toml,target=rust-toolchain.toml,readonly \
    \
    --mount=type=bind,source=stellwerk-api/src,target=stellwerk-api/src,readonly \
    --mount=type=bind,source=stellwerk-api/Cargo.toml,target=stellwerk-api/Cargo.toml,readonly \
    \
    --mount=type=bind,source=stellwerk-common/src,target=stellwerk-common/src,readonly \
    --mount=type=bind,source=stellwerk-common/Cargo.toml,target=stellwerk-common/Cargo.toml,readonly \
    \
    --mount=type=bind,source=stellwerk-db/src,target=stellwerk-db/src,readonly \
    --mount=type=bind,source=stellwerk-db/Cargo.toml,target=stellwerk-db/Cargo.toml,readonly \
    --mount=type=bind,source=stellwerk-db/.cargo,target=stellwerk-db/.cargo,readonly \
    --mount=type=bind,source=stellwerk-db/.sqlx,target=stellwerk-db/.sqlx,readonly \
    --mount=type=bind,source=stellwerk-db/migrations,target=stellwerk-db/migrations,readonly \
    \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/stellwerk-api /bin/stellwerk-api
EOF

FROM debian:bookworm-slim AS final

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=rust-build /bin/stellwerk-api /bin/

CMD ["/bin/stellwerk-api"]
