use Mix.Config

config :logger, level: :info

config :simple_budget, SimpleBudget.Repo,
  url: System.get_env("DATABASE_URL"),
  pool_size: 6

config :simple_budget, SimpleBudgetWeb.Endpoint,
  secret_key_base: System.get_env("SECRET_KEY_BASE"),
  cache_static_manifest: "priv/static/cache_manifest.json"

config :simple_budget,
  cookie_signing_salt: System.get_env("COOKIE_SIGNING_SALT"),
  cookie_encryption_salt: System.get_env("COOKIE_ENCRYPTION_SALT")
