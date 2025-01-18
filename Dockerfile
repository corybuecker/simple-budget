FROM node AS frontend
RUN mkdir -p /app/static
COPY input.css tailwind.config.js /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/typography @tailwindcss/forms @tailwindcss/aspect-ratio @tailwindcss/container-queries
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust:1.84.0-slim AS builder
RUN mkdir /app
WORKDIR /app
COPY Cargo.toml Cargo.lock /app
COPY src /app/src
COPY templates /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN cargo build --release

FROM rust:1.84.0-slim
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
RUN chown 1000:1000 /app/simple-budget 
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
