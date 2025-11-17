FROM node@sha256:a84dedcbbcf651db49178dd876bad1ea63bc6770891337a05246991bceca55a7 AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:cd34b27bc6df5450e4952075dc6bd3881a1aeb8f5d0478cd75c2160ca47e2182 AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release

FROM debian@sha256:01a723bf5bfb21b9dda0c9a33e0538106e4d02cce8f557e118dd61259553d598
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
WORKDIR /app
RUN chmod 700 /app/simple-budget
COPY templates /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN chown -R 1000:1000 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
