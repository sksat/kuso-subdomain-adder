FROM ghcr.io/sksat/cargo-chef-docker:1.54.0-slim as planner
WORKDIR chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM ghcr.io/sksat/cargo-chef-docker:1.54.0-slim as cacher
WORKDIR chef
COPY --from=planner /chef/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.53.0 as builder
WORKDIR build
ADD . .
COPY --from=cacher /chef/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
