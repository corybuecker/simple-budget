FROM elixir:1.9.0-alpine AS builder
RUN apk update && apk add git make build-base
COPY mix.exs mix.lock /app/
WORKDIR /app
ENV MIX_ENV=prod
RUN mix local.hex --force && \
  mix local.rebar --force
RUN mix deps.get && \
  mix deps.compile
RUN apk del make build-base && \
  rm -f /var/cache/apk/*

FROM node:11.6 AS assets
COPY assets /assets
WORKDIR /assets
RUN npm install
COPY --from=builder /app/deps /deps
RUN npm run deploy

FROM builder
COPY config /app/config
COPY lib /app/lib
COPY priv /app/priv
COPY --from=assets /priv/static /app/priv/static
ENV MIX_ENV=prod COOKIE_SIGNING_SALT=justneedassets
RUN mix compile && \
  mix phx.digest

CMD ["mix", "phx.server"]

LABEL maintainer="cory.buecker@gmail.com"
