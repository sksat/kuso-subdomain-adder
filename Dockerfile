FROM ghcr.io/sksat/cargo-chef-docker:1.90.0-bullseye@sha256:b95afab481b1037908a39464b246ee3907e8bfc066c21cf36236a5c2437a00a2 as cargo-chef

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
