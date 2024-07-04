FROM rust:1.79.0 AS backend_builder
RUN mkdir -p /build
COPY Cargo.toml /build/
COPY src /build/src
WORKDIR /build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release
RUN --mount=type=cache,target=/build/target cp /build/target/release/simple-budget /build/simple-budget

FROM node:alpine AS frontend_builder
RUN mkdir /build
COPY static /build/static
COPY src/templates /build/src/templates
COPY tailwind.config.js /build
WORKDIR /build
RUN npm install tailwindcss @tailwindcss/container-queries @tailwindcss/forms
RUN npx tailwindcss -i static/app.css -o app.css

FROM rust:1.79.0-slim
COPY --from=backend_builder /build/simple-budget /app/simple-budget
COPY src/templates /app/src/templates
COPY --from=frontend_builder /build/app.css /app/static/
WORKDIR /app
CMD ["/app/simple-budget"]