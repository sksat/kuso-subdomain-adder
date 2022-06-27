FROM ghcr.io/sksat/cargo-chef-docker:1.61.0-bullseye@sha256:30a8c5f55e8a0b9bb660a49ce29b0bad98adb84dd143dd9eef810fa0b8650bcc as cargo-chef

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
