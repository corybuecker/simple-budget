use Mix.Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :simple_budget, SimpleBudgetWeb.Endpoint,
  http: [port: 4001],
  server: false,
  secret_key_base:
    "SECRET_KEY_BASESECRET_KEY_BASESECRET_KEY_BASESECRET_KEY_BASESECRET_KEY_BASESECRET_KEY_BASE"

# Print only warnings and errors during test
config :logger, level: :warn

# Configure your database
config :simple_budget, SimpleBudget.Repo,
  url: System.get_env("TEST_DATABASE_URL"),
  pool: Ecto.Adapters.SQL.Sandbox

config :argon2_elixir, t_cost: 2, m_cost: 8

config :simple_budget,
  token_key: "development-use-only",
  google_client_id: "test"

config :simple_budget,
  cookie_signing_salt: "COOKIE_SIGNING_SALT",
  cookie_encryption_salt: "COOKIE_ENCRYPTION_SALT"
