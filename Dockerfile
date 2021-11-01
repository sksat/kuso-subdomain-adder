FROM ghcr.io/sksat/cargo-chef-docker:1.56.0-bullseye@sha256:88c8d857d3e41454aa60e1b743d5ea01d4ae0b5c16374b000e2ee02804823566 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:2510b5f6a5ca842b863b298ef6b1e423b7b0f1a5343db5e94ffc1943d0e70098
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
