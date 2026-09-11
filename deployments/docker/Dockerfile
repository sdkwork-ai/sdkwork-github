FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-api-github-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-api-github-standalone-gateway /usr/local/bin/sdkwork-api-github-standalone-gateway
ENV SDKWORK_GITHUB_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:4100
EXPOSE 4100
CMD ["sdkwork-api-github-standalone-gateway"]
