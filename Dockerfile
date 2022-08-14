FROM ghcr.io/sksat/cargo-chef-docker:1.63.0-bullseye@sha256:2915417d932c55bcd5183592e301c33a19bb2be7bdc5322bdad41f6d026a5577 as cargo-chef

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

FROM gcr.io/distroless/cc@sha256:1dc7ae628f0308f77dac8538b4b246453ac3636aa5ba22084e3d22d59a7f3cca
WORKDIR app
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder", "srv"]
