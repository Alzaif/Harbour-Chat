# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS api-build
WORKDIR /app/api
COPY api/Cargo.toml api/Cargo.lock* ./
COPY api/src ./src
COPY api/migrations ./migrations
RUN cargo build --release

FROM node:22-alpine AS web-build
WORKDIR /app/web
ARG VITE_HARBOUR_SHELL_URL=https://harbour.local
ENV VITE_HARBOUR_SHELL_URL=$VITE_HARBOUR_SHELL_URL

COPY web/package.json web/package-lock.json* ./
RUN npm ci 2>/dev/null || npm install

COPY web/ ./
RUN npm run build

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends nginx ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV PORT=3000
ENV TRUST_GATEWAY_HEADERS=true
ENV CHAT_DB_PATH=/data/chat.db
ENV CHAT_DATA_DIR=/data
ENV RUST_LOG=info

COPY --from=api-build /app/api/target/release/harbour-chat-api /harbour-chat-api
COPY --from=web-build /app/web/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/sites-available/default
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh && mkdir -p /data \
    && ln -sf /etc/nginx/sites-available/default /etc/nginx/sites-enabled/default

EXPOSE 80
VOLUME ["/data"]
ENTRYPOINT ["/entrypoint.sh"]
