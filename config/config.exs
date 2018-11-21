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
  url: [host: "localhost"],
  secret_key_base:
    "must_be_set_in_envrionment_outside_developmentmust_be_set_in_envrionment_outside_development",
  render_errors: [view: SimpleBudgetWeb.ErrorView, accepts: ~w(html json)],
  pubsub: [name: SimpleBudget.PubSub, adapter: Phoenix.PubSub.PG2]

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :simple_budget, env: Mix.env()

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{Mix.env()}.exs"
