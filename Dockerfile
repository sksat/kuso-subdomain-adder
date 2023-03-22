FROM ghcr.io/sksat/cargo-chef-docker:1.68.0-bullseye@sha256:ae9af2b905a1a3b977625d7d4aefae36bd2e3b93e481aa4eec6eaf8e82e2b517 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:fb402c45f3ef485ccd56ca2af2a58615fc47c4978bb3004e8663a83456791f48
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
