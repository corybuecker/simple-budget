FROM node@sha256:d2b6b5aedb5b729f68ee1129e0f5a5d4713d93f82448249e82241876d8e8d86e AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:e090f7b4adf86191313dba91260351d7f5e15cac0fe34f26706a805c0cb9641f AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
COPY templates /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN cargo build --release

FROM debian@sha256:6d87375016340817ac2391e670971725a9981cfc24e221c47734681ed0f6c0f5
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
RUN chown 1000:1000 /app/simple-budget 
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
