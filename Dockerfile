FROM ghcr.io/sksat/cargo-chef-docker:1.66.0-bullseye@sha256:bedc2a054bfbfe61a566127a6406533399ddb3643889ae7e41c08899260f4dc9 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:101c26286ea36b68200ff94cf95ca9dbde3329c987738cba3ba702efa3465f6f
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
