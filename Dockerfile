FROM elixir:alpine
COPY / /app
WORKDIR /app
ENV MIX_ENV=prod PORT=4000
RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix deps.get && \
    MIX_ENV=prod mix compile && \
    mix phx.digest

CMD ["mix", "phx.server"]

LABEL maintainer="cory.buecker@gmail.com"
