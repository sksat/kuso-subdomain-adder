FROM ghcr.io/sksat/cargo-chef-docker:1.65.0-bullseye@sha256:10359dd391fc6197df059509be075f781351854810069206abbba5685c5c022d as cargo-chef

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
