FROM node@sha256:701c8a634cb3ddbc1dc9584725937619716882525356f0989f11816ba3747a22 AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates_legacy /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:3329e2de3e9ff2d58da56e95ef99a3180a4e76336a676f3fe2b88f0b0d6bcfbf AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
COPY templates_legacy /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN cargo build --release

FROM debian@sha256:6d87375016340817ac2391e670971725a9981cfc24e221c47734681ed0f6c0f5
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
RUN chown 1000:1000 /app/simple-budget
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
