FROM ghcr.io/sksat/cargo-chef-docker:1.56.0-bullseye@sha256:29fb2b03a149a25deced4dcf0a07a67c430b26fa501a828b785824ceeeb80d7d as cargo-chef

FROM cargo-chef as planner
WORKDIR chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM cargo-chef as builder
WORKDIR build
COPY --from=planner /chef/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc@sha256:53ae81ce96dfebf79d515e89af05c5cea25ad42618ceb77be7b6160a3e2d32da
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
