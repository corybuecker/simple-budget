FROM rust:1.84.0-slim AS builder

RUN apt-get update
RUN apt-get install -y curl

RUN mkdir /app
WORKDIR /app

COPY Cargo.toml Cargo.lock /app

RUN curl -o- -fsSL https://deb.nodesource.com/setup_23.x | bash
RUN apt-get install -y nodejs

COPY input.css tailwind.config.js /app
COPY static /app/static

RUN npm install tailwindcss @tailwindcss/typography @tailwindcss/forms @tailwindcss/aspect-ratio @tailwindcss/container-queries

COPY src /app/src
COPY templates /app/templates
RUN cargo build --release
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust:1.84.0-slim

COPY --from=builder /app/target/release/simple-budget /app/simple-budget

RUN chown 1000:1000 /app/simple-budget 
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
