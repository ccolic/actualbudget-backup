FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
LABEL org.opencontainers.image.source=https://github.com/ccolic/actualbudget-backup
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/actualbudget-backup ./actualbudget-backup

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/actualbudget-backup /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/actualbudget-backup"]
