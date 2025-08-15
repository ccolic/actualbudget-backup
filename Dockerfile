FROM rust:1 AS chef
LABEL org.opencontainers.image.source=https://github.com/ccolic/actualbudget-backup
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release

FROM debian:13 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/actualbudget-backup /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/actualbudget-backup"]
