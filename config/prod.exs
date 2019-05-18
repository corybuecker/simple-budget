use Mix.Config

config :logger, level: :info

config :simple_budget, SimpleBudgetWeb.Endpoint,
  cache_static_manifest: "priv/static/cache_manifest.json",
  url: [host: System.get_env("SSL_HOST")]

config :simple_budget, SimpleBudget.Repo,
  url: System.get_env("DATABASE_URL"),
  pool_size: 6
