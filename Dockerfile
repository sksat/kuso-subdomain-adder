FROM ghcr.io/sksat/cargo-chef-docker:1.68.2-bullseye@sha256:b9a4df395b55c19d0a2239f8f08d7cf64e26ecc2f3f5bdd97551f235967d6c40 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:8aad707f96620ee89e27febef51b01c6ff244277a3560fcfcfbe68633ef09193
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
