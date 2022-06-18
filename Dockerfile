FROM ghcr.io/sksat/cargo-chef-docker:1.61.0-bullseye@sha256:637e8718937fd4a4e321eb2732bd6e0043565749ce9002be78594c0c440e737d as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:1b82fde9abdd6b83077fa99af6b7bb93fcde1e93325eb00bfb814d5068ce60d9
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
