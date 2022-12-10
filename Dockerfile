FROM ghcr.io/sksat/cargo-chef-docker:1.65.0-bullseye@sha256:182df8097a18ceed2c088eb372cf6f39c0848937ff24c004dc0c94dba23c8414 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:442431d783e435ff0954d2a8214231e3da91ce392c65c1138c0bcdbc77420116
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
