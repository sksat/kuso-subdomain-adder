FROM ghcr.io/sksat/cargo-chef-docker:1.76.0-bullseye@sha256:26d06c579d4abbc2099cec150223b7049c6c6a03ce080ee92de5c3bdc14e13d2 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:396891e37c26c8ea032aef368c806f64c950d19cc578fdab2b0093710a036895
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
