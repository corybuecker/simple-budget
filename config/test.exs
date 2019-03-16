use Mix.Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :simple_budget, SimpleBudgetWeb.Endpoint,
  http: [port: 4001],
  server: false

# Print only warnings and errors during test
config :logger, level: :warn

# Configure your database
config :simple_budget, SimpleBudget.Repo,
  username: "postgres",
  password: "postgres",
  database: "simple_budget_test",
  hostname: "localhost",
  pool: Ecto.Adapters.SQL.Sandbox

config :argon2_elixir, t_cost: 2, m_cost: 8

config :simple_budget,
  token_key: "development-use-only",
  google_client_id: "test"
