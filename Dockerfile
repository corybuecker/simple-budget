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

FROM scratch
COPY --from=backend_builder /build/simple-budget /simple-budget
COPY src/templates /src/templates
COPY controllers /static/controllers
COPY src/favicon.svg /static/favicon.svg
COPY src/favicon.png /static/favicon.png
COPY etc_passwd /etc/passwd
COPY --from=frontend_builder /build/app.css /static/app.css
COPY --from=frontend_builder /build/src/google.css /static/google.css
COPY --from=backend_builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=backend_builder /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6
COPY --from=backend_builder /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6
COPY --from=backend_builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
USER 65534
CMD ["/simple-budget"]
