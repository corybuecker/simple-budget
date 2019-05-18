# This file is responsible for configuring your application
# and its dependencies with the aid of the Mix.Config module.
#
# This configuration file is loaded before any dependency and
# is restricted to this project.

# General application configuration
use Mix.Config

config :simple_budget,
  ecto_repos: [SimpleBudget.Repo]

# Configures the endpoint
config :simple_budget, SimpleBudgetWeb.Endpoint,
  http: [port: 4000],
  secret_key_base: System.get_env("SECRET_KEY_BASE"),
  render_errors: [view: SimpleBudgetWeb.ErrorView, accepts: ~w(html json)],
  pubsub: [name: SimpleBudget.PubSub, adapter: Phoenix.PubSub.PG2]

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :simple_budget, env: Mix.env()

config :simple_budget,
  token_key: System.get_env("TOKEN_KEY"),
  google_client_id: System.get_env("GOOGLE_CLIENT_ID"),
  sso_enabled: true

config :simple_budget,
  cookie_signing_salt: System.get_env("COOKIE_SIGNING_SALT"),
  cookie_encryption_salt: System.get_env("COOKIE_ENCRYPTION_SALT")

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{Mix.env()}.exs"
