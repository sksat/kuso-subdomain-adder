FROM ghcr.io/sksat/cargo-chef-docker:1.66.1-bullseye@sha256:ed0c79a4c98cea639baac2b193779ee849a41515dfabf48812b47e574e7ca7b0 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:20ec142cb2763f73126ed6e6462f74777fbd8a49e1037cb1b0a5118749117954
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
