FROM node:9-alpine as assets
RUN apk update && apk add yarn python
ADD www/package.json www/yarn.lock /tmp/
RUN cd /tmp && yarn
RUN mkdir -p /app/www && cd /app/www && ln -s /tmp/node_modules
COPY api /app/api
COPY www /app/www
WORKDIR /app/www
RUN yarn run webpack --config webpack.prod.js

FROM elixir:1.6-alpine
COPY --from=assets /app/api /app
WORKDIR /app
ENV MIX_ENV=prod PORT=4000
RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix deps.get && \
    mix phx.digest
CMD ["mix", "phx.server"]

MAINTAINER Cory Buecker <cory.buecker@gmail.com>
