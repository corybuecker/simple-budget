FROM node@sha256:ccfc02deb6abb1b70b6ef21d3d93b3f671c0de6f463ff331cf0ea0a28ad875c9 AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:f2a0f2b3529c9bbbf5479d131611451a3cc3956d9a11374d6d4ba96f059c1dce AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release

FROM debian@sha256:55a15a112b42be10bfc8092fcc40b6748dc236f7ef46a358d9392b339e9d60e8
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
WORKDIR /app
RUN chmod 700 /app/simple-budget
COPY templates /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN chown -R 1000:1000 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
