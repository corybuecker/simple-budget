FROM node AS frontend_builder
RUN mkdir /build
COPY src/templates /build/src/templates
COPY src/input.css /build/src/input.css
COPY src/google.css /build/src/google.css
COPY tailwind.config.js /build
WORKDIR /build
RUN npm install tailwindcss @tailwindcss/container-queries @tailwindcss/forms
RUN npx tailwindcss -i src/input.css -o app.css
COPY controllers /build/controllers
RUN gzip -k app.css controllers/*.js

# Avoid doing anything in these steps outside of COPY
FROM rust:1.82.0-alpine
COPY --from=frontend_builder /build/controllers /static/controllers
COPY --from=frontend_builder /build/app.css /static/app.css
COPY --from=frontend_builder /build/app.css.gz /static/app.css.gz
COPY --from=frontend_builder /build/src/google.css /static/google.css
COPY src/favicon.png /static/favicon.png
COPY etc_passwd /etc/passwd
COPY src/templates /src/templates

COPY simple-budget /simple-budget

USER 65534
CMD ["/simple-budget"]
