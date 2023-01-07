FROM ghcr.io/sksat/cargo-chef-docker:1.66.0-bullseye@sha256:bedc2a054bfbfe61a566127a6406533399ddb3643889ae7e41c08899260f4dc9 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:107c9b25dcdb5ad8fec3e047cdd8e69c26f6d4505cbfc652ce18644e745b6f26
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
