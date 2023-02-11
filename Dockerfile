FROM ghcr.io/sksat/cargo-chef-docker:1.67.0-bullseye@sha256:cb2de3d614456b83f7834360cea3408451728f977260f1df8b3d778a1b5c6487 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:a37569bbd5a092cae7839318fcfba6d5159d1d12a1b2216c2294219d71bb22a7
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
