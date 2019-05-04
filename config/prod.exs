use Mix.Config

config :logger, level: :info

config :simple_budget, SimpleBudget.Repo,
  url: System.get_env("DATABASE_URL"),
  pool_size: 2

config :simple_budget, SimpleBudgetWeb.Endpoint,
  force_ssl: [hsts: true, rewrite_on: [:x_forwarded_proto]],
  http: [port: System.get_env("PORT")],
  load_from_system_env: true,
  url: [scheme: "https", host: "budget.bueckered.com", port: 443],
  secret_key_base: System.get_env("SECRET_KEY_BASE"),
  cache_static_manifest: "priv/static/cache_manifest.json"
