FROM lukemathwalker/cargo-chef:latest-rust-1.69.0-bullseye AS chef
ARG SQLX_CLI_VERSION=0.6.2
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y  \
      cmake \
      pkg-config \
      libssl-dev \
      git \
      clang \
      openssl \
    && cargo install --version=$SQLX_CLI_VERSION sqlx-cli --no-default-features --features native-tls,postgres \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS  builder
WORKDIR /app
ENV SQLX_OFFLINE=true \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-20221219-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

FROM runtime AS project
ARG PROJECT_NAME=battlemon-ethereum
WORKDIR /app
COPY --from=builder /app/target/release/$PROJECT_NAME ./app
COPY --from=builder /app/config/ ./
ENTRYPOINT ["./app"]
