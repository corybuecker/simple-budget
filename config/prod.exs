use Mix.Config

config :logger, level: :info

config :simple_budget, SimpleBudgetWeb.Endpoint,
  cache_static_manifest: "priv/static/cache_manifest.json",
  force_ssl: [rewrite_on: [:x_forwarded_proto]]

config :simple_budget, SimpleBudget.Repo,
  url: System.get_env("DATABASE_URL"),
  pool_size: 6
