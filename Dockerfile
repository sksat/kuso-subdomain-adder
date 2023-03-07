FROM ghcr.io/sksat/cargo-chef-docker:1.67.1-bullseye@sha256:cdbfc5e1836178330045e604065c7904b5de89200eb31e0825a8e9b51c7be90c as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:22439f05b5ba66e8d17088c5dda6775e91f62e3f48068830bd8a5c21d1cddc68
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
