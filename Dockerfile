FROM ghcr.io/sksat/cargo-chef-docker:1.62.0-bullseye@sha256:f76997f2b50b47be14af7eae159a34f1afc22ebd7d017bdbce13547465cc6a7b as cargo-chef

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
