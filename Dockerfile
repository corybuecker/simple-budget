FROM rust:1.81.0 AS backend_builder

RUN mkdir -p /build
WORKDIR /build
COPY Cargo.toml /build/
RUN mkdir -p /build/src
RUN echo "fn main() {}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/simple-budget /build/simple-budget

FROM node:alpine AS frontend_builder
RUN mkdir /build
COPY src/templates /build/src/templates
COPY src/input.css /build/src/input.css
COPY src/google.css /build/src/google.css
COPY tailwind.config.js /build
WORKDIR /build
RUN npm install tailwindcss @tailwindcss/container-queries @tailwindcss/forms
RUN npx tailwindcss -i src/input.css -o app.css

FROM rust:1.81.0-slim
COPY --from=backend_builder /build/simple-budget /app/simple-budget
COPY src/templates /app/src/templates
COPY controllers /app/static/controllers
COPY --from=frontend_builder /build/app.css /app/static/app.css
COPY --from=frontend_builder /build/src/google.css /app/static/google.css

WORKDIR /app
CMD ["/app/simple-budget"]
