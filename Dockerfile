FROM elixir:1.7-alpine AS builder
RUN apk update && apk add git
COPY mix.exs mix.lock /app/
WORKDIR /app
ENV MIX_ENV=prod
RUN mix local.hex --force && \
    mix local.rebar --force
RUN mix deps.get && \
    mix deps.compile

FROM node:10.12 AS assets-base
COPY assets/package.json assets/package-lock.json /assets-base/
WORKDIR /assets-base
RUN npm install

FROM assets-base AS assets
COPY assets /assets
WORKDIR /assets
RUN ln -s /assets-base/node_modules .
COPY --from=builder /app/deps /deps
RUN mv /assets/js/accounts /assets/js/Accounts && \
    mv /assets/js/adjustments /assets/js/Adjustments && \
    mv /assets/js/goals /assets/js/Goals && \
    mv /assets/js/savings /assets/js/Savings && \
    rm -rf /assets/js/elm-stuff && npm run deploy

FROM builder
COPY config /app/config
COPY lib /app/lib
COPY priv /app/priv
COPY --from=assets /priv/static /app/priv/static
ENV MIX_ENV=prod PORT=4001
RUN mix compile && \
    mix phx.digest

CMD ["mix", "phx.server"]

LABEL maintainer="cory.buecker@gmail.com"
