FROM node:9-alpine as assets
RUN apk update && apk add yarn python
ADD assets/package.json assets/yarn.lock /tmp/
RUN cd /tmp && yarn
RUN mkdir -p /app/assets && cd /app/assets && ln -s /tmp/node_modules
COPY / /app/
WORKDIR /app/assets
RUN yarn run webpack --config webpack.prod.js

FROM elixir:1.6-alpine
COPY --from=assets /app /app
WORKDIR /app
RUN rm -rf assets
ENV MIX_ENV=prod PORT=4000
RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix deps.get && \
    mix phx.digest
CMD ["mix", "phx.server"]

MAINTAINER Cory Buecker <cory.buecker@gmail.com>
