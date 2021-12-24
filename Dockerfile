FROM ghcr.io/sksat/cargo-chef-docker:1.57.0-bullseye@sha256:095bcbf9b8de602ece26d90e3e35cb3871e866c5b88b2667c4a651bfe2676a0d as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:0090fc97e9cbc060fb5eb1bcee153997942096f51006bef2200233d762b2bb0e
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
