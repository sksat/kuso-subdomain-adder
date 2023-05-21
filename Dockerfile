FROM ghcr.io/sksat/cargo-chef-docker:1.69.0-bullseye@sha256:ff3acddafe127faad9c9c80ef9f4a7bbeca5c2527091c81636173cc1c089d4af as cargo-chef

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
