FROM rust:latest as builder
WORKDIR build
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/kuso-subdomain-adder /app
CMD ["/app/kuso-subdomain-adder"]
