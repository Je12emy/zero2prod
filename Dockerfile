# We use the latest Rust stable release as base image
FROM lukemathwalker/cargo-chef:latest-rust-1 as chef
# Let's switch our working directory to `app` (equivalent to `cd app`) # The `app` folder will be created for us by Docker in case it does not # exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration 
RUN apt update && apt install lld clang -y

FROM chef as planner
# Copy all files from our working environment to our Docker image
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
COPY . .
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
