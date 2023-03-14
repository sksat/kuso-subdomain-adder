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

FROM gcr.io/distroless/cc@sha256:fb402c45f3ef485ccd56ca2af2a58615fc47c4978bb3004e8663a83456791f48
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
