FROM node@sha256:c4bfed36421c310d1fbb6dc51faf98065768fbc1c2c1ddd554813ecaa81bb2db AS frontend
RUN mkdir -p /app/static
COPY input.css /app
COPY templates /app/templates
COPY static /app/static
WORKDIR /app
RUN npm install tailwindcss @tailwindcss/cli
RUN npx tailwindcss -i input.css -o static/app.css

FROM rust@sha256:51c04d7a2b38418ba23ecbfb373c40d3bd493dec1ddfae00ab5669527320195e AS builder
RUN mkdir -p /app/src
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN echo "fn main(){}" > /app/src/main.rs
RUN cargo build --release
COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release

FROM debian@sha256:3615a749858a1cba49b408fb49c37093db813321355a9ab7c1f9f4836341e9db
COPY --from=builder /app/target/release/simple-budget /app/simple-budget
WORKDIR /app
RUN chmod 700 /app/simple-budget
COPY templates /app/templates
COPY static /app/static
COPY --from=frontend /app/static/app.css /app/static/app.css
RUN chown -R 1000:1000 /app/simple-budget
USER 1000
CMD ["/app/simple-budget"]
