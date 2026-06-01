FROM node:lts-trixie@sha256:f072159a6b98a624e09f2c4815fe473217fc019a97524fd593059c8a4ad5a05d AS frontend
RUN mkdir -p /app/static
COPY assets /app/assets
COPY templates /app/templates
COPY package.json /app/package.json
COPY pnpm-lock.yaml /app/pnpm-lock.yaml
COPY pnpm-workspace.yaml /app/pnpm-workspace.yaml
COPY vite.config.ts /app/vite.config.ts
WORKDIR /app
RUN npm install -g pnpm@11.2.2
RUN pnpm install --frozen-lockfile
RUN pnpm build

FROM rust:1.95.0-trixie@sha256:0861191076afc8e2dfcf0bec6ad6c2dec8494b3a1e9249729e1989690afed5ec AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release

FROM debian:trixie-slim@sha256:4ffb3a1511099754cddc70eb1b12e50ffdb67619aa0ab6c13fcd800a78ef7c7a
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
WORKDIR /app
RUN chmod 700 /app/simple-budget
COPY templates /app/templates
COPY --from=frontend /app/static/index.mjs /app/static/index.mjs
COPY --from=frontend /app/static/index.css /app/static/index.css
RUN chown -R 1000:1000 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
