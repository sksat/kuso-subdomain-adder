FROM ghcr.io/sksat/cargo-chef-docker:1.66.1-bullseye@sha256:4165d67fa5ddd8616cbcabe9bdf1235eb279f964b0d4abe1b7abbdab6a46e36f as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:20ec142cb2763f73126ed6e6462f74777fbd8a49e1037cb1b0a5118749117954
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
