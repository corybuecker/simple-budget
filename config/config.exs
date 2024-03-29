# This file is responsible for configuring your application
# and its dependencies with the aid of the Config module.
#
# This configuration file is loaded before any dependency and
# is restricted to this project.

# General application configuration
import Config

config :simple_budget,
  ecto_repos: [SimpleBudget.Repo]

# Configures the endpoint
config :simple_budget, SimpleBudgetWeb.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: SimpleBudgetWeb.ErrorHTML, json: SimpleBudgetWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: SimpleBudget.PubSub,
  live_view: [signing_salt: "OjTJ0PMC"]

# Configures the mailer
#
# By default it uses the "Local" adapter which stores the emails
# locally. You can see the emails in your browser, at "/dev/mailbox".
#
# For production it's recommended to configure a different adapter
# at the `config/runtime.exs`.
config :simple_budget, SimpleBudget.Mailer, adapter: Swoosh.Adapters.Local

# Configure esbuild (the version is required)
config :esbuild,
  version: "0.18.7",
  default: [
    args:
      ~w(js/app.js --bundle --target=es2017 --format=esm --outdir=../priv/static/assets --external:/fonts/* --external:/images/* --external:phoenix --external:phoenix_html --external:phoenix_live_view --external:topbar --external:vanillajs-datepicker --external:@hotwired/strada --external:@hotwired/stimulus --external:@hotwired/turbo),
    cd: Path.expand("../assets", __DIR__),
    env: %{"NODE_PATH" => Path.expand("../deps", __DIR__)}
  ]

# Configure tailwind (the version is required)
config :tailwind,
  version: "3.2.7",
  default: [
    args: ~w(
      --config=tailwind.config.js
      --input=css/app.css
      --output=../priv/static/assets/app.css
    ),
    cd: Path.expand("../assets", __DIR__)
  ]

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason

config :simple_budget, SimpleBudget.Goals, datetime_adapter: SimpleBudget.Utilities.DateTime

config :simple_budget, :cluster, true

config :rollbax,
  access_token: System.get_env("ROLLBAR"),
  enable_crash_reports: true

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{config_env()}.exs"
