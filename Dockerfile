FROM node@sha256:8369522c586f6cafcf77e44630e7036e4972933892f8b45e42d9baeb012d521c AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:749d5f12aa5f38ebf81012a0385b8e6adcb7b6e8f494961d559e8a7264803d4f AS builder
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

FROM debian@sha256:d42b86d7e24d78a33edcf1ef4f65a20e34acb1e1abd53cabc3f7cdf769fc4082
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
RUN chown 1000:1000 /app/simple-budget 
RUN chmod 700 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
