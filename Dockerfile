FROM node:lts-trixie@sha256:f7d34e58713740f9eef9092c0bd6ff10369d132f7238399a4b270f16d47fa608 AS frontend
RUN mkdir -p /app/static
COPY assets /app/assets
COPY templates /app/templates
COPY package.json /app/package.json
COPY pnpm-lock.yaml /app/pnpm-lock.yaml
COPY pnpm-workspace.yaml /app/pnpm-workspace.yaml
COPY vite.config.ts /app/vite.config.ts
WORKDIR /app
RUN npm install -g pnpm@11.11.0
RUN pnpm install --frozen-lockfile
RUN pnpm build

FROM rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07 AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release

FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
WORKDIR /app
RUN chmod 700 /app/simple-budget
COPY templates /app/templates
COPY --from=frontend /app/static/index.mjs /app/static/index.mjs
COPY --from=frontend /app/static/index.css /app/static/index.css
RUN chown -R 1000:1000 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
