FROM rust:1.98.1-slim-bookworm AS build
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p quorumscope-cli

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 quorumscope
COPY --from=build /src/target/release/quorumscope /usr/local/bin/quorumscope
USER quorumscope
CMD ["quorumscope", "serve"]
