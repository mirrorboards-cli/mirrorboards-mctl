# WYGENEROWANE przez mctl forge (kind: vite-static) — nie edytować ręcznie.
#
# Trzy warstwy: kraty WASM (przebudowują się TYLKO przy zmianie w nich),
# build JS, statyczny serwer. Wartości PUBLIC_* są wklejane w build —
# ustawienie ich w kontenerze nie ma skutku.
FROM rust:bookworm AS wasm
WORKDIR /workspace
RUN rustup target add wasm32-unknown-unknown \
 && curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
COPY . .

{{WASM_STAGES}}FROM node:22-slim AS builder
RUN corepack enable && corepack prepare pnpm@10.15.0 --activate
WORKDIR /workspace
COPY . .
{{WASM_COPIES}}RUN pnpm install --no-frozen-lockfile
WORKDIR /workspace/{{APP_DIR}}
{{PUBLIC_LINES}}RUN pnpm build

FROM ghcr.io/static-web-server/static-web-server:2 AS runtime
ENV SERVER_PORT={{PORT}}
ENV SERVER_ROOT=/app/dist
# Routing historii: nieznane ścieżki wracają do wejścia SPA.
ENV SERVER_FALLBACK_PAGE=/app/dist/index.html
# `.well-known` jest katalogiem standardowym (RFC 8615), a serwer domyślnie
# ukrywa ścieżki z kropką i oddaje na nie fallback SPA z mylnym typem treści.
ENV SERVER_IGNORE_HIDDEN_FILES=false
{{ENV_LINES}}COPY --from=builder /workspace/{{APP_DIR}}/dist /app/dist
{{RUNTIME_COPIES}}EXPOSE {{PORT}}
