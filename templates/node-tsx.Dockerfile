# WYGENEROWANE przez mctl forge (kind: node-tsx) — nie edytować ręcznie.
FROM node:24-slim
RUN npm install -g pnpm@10.15.0
WORKDIR /workspace
COPY . .
RUN pnpm install
ENV PORT={{PORT}}
{{ENV_LINES}}EXPOSE {{PORT}}
USER node
WORKDIR /workspace/{{APP_DIR}}
ENTRYPOINT {{CMD_JSON}}
