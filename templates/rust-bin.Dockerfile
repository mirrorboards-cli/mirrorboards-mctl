# WYGENEROWANE przez mctl forge (kind: rust-bin) — nie edytować ręcznie.
#
# Granica cache'u leży między kratami z crates.io a kodem workspace'u:
# `cargo chef cook` kompiluje zależności zewnętrzne w warstwie, która
# unieważnia się TYLKO przy zmianie manifestów/locka. Typowy commit
# (router/core/appka) przebudowuje wyłącznie kraty workspace'u.
# Wariant BOOKWORM, nie „latest-rust-1": obraz chefa jedzie dziś na nowszym
# Debianie niż runtime, więc binarka żądałaby GLIBC 2.38, którego bookworm-slim
# nie ma — build przechodzi, a kontener wywala się przy starcie.
FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /workspace

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p {{BIN}}

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/{{BIN}} /usr/local/bin/{{BIN}}
ENV PORT={{PORT}}
{{ENV_LINES}}EXPOSE {{PORT}}
ENTRYPOINT ["/usr/local/bin/{{BIN}}"]
