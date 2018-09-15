FROM node:alpine as assets
RUN apk update && apk add yarn python git
ADD assets/package.json assets/yarn.lock /tmp/
RUN cd /tmp && yarn install --pure-lockfile
RUN mkdir -p /app/assets && cd /app/assets && ln -s /tmp/node_modules
COPY / /app/
WORKDIR /app/assets
RUN yarn run webpack --mode production

FROM elixir:alpine
COPY --from=assets /app /app
WORKDIR /app
RUN rm -rf assets
ENV MIX_ENV=prod PORT=4000
RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix deps.get && \
    MIX_ENV=prod mix compile && \
    mix phx.digest

CMD ["mix", "phx.server"]

LABEL maintainer="cory.buecker@gmail.com"
