# syntax=docker/dockerfile:1

FROM rustlang/rust:nightly-bookworm-slim AS rust-build
WORKDIR /app

RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml,readonly \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock,readonly \
    --mount=type=bind,source=rust-toolchain.toml,target=rust-toolchain.toml,readonly \
    \
    --mount=type=bind,source=socialmediathingnametbd-api/src,target=socialmediathingnametbd-api/src,readonly \
    --mount=type=bind,source=socialmediathingnametbd-api/Cargo.toml,target=socialmediathingnametbd-api/Cargo.toml,readonly \
    \
    --mount=type=bind,source=socialmediathingnametbd-common/src,target=socialmediathingnametbd-common/src,readonly \
    --mount=type=bind,source=socialmediathingnametbd-common/Cargo.toml,target=socialmediathingnametbd-common/Cargo.toml,readonly \
    \
    --mount=type=bind,source=socialmediathingnametbd-db/src,target=socialmediathingnametbd-db/src,readonly \
    --mount=type=bind,source=socialmediathingnametbd-db/Cargo.toml,target=socialmediathingnametbd-db/Cargo.toml,readonly \
    --mount=type=bind,source=socialmediathingnametbd-db/.cargo,target=socialmediathingnametbd-db/.cargo,readonly \
    --mount=type=bind,source=socialmediathingnametbd-db/.sqlx,target=socialmediathingnametbd-db/.sqlx,readonly \
    --mount=type=bind,source=socialmediathingnametbd-db/migrations,target=socialmediathingnametbd-db/migrations,readonly \
    \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/socialmediathingnametbd-api /bin/socialmediathingnametbd-api
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

COPY --from=rust-build /bin/socialmediathingnametbd-api /bin/

CMD ["/bin/socialmediathingnametbd-api"]
