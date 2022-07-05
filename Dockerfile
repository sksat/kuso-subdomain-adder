FROM ghcr.io/sksat/cargo-chef-docker:1.62.0-bullseye@sha256:97c18c33027b17c79d2e654eee15a77781e549e9e72aec3a61a137c081f96073 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:bac217056540e5330875164e5d6e29b2fcd2725ed0994332b6a8650d57ddd94d
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
