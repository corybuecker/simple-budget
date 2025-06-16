FROM node@sha256:c332080545f1de96deb1c407e6fbe9a7bc2be3645e127845fdcce57a7af3cf56 AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:25038aa450210c53cf05dbf7b256e1df1ee650a58bb46cbc7d6fa79c1d98d083 AS builder
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

FROM debian@sha256:0d8498a0e9e6a60011df39aab78534cfe940785e7c59d19dfae1eb53ea59babe
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
RUN chown 1000:1000 /app/simple-budget 
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
