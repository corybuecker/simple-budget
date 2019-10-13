use Mix.Config

# Configure your database
config :simple_budget, SimpleBudget.Repo,
  url:
    System.get_env(
      "TEST_DATABASE_URL",
      "ecto://postgres:postgres@localhost:5432/simple_budget_test"
    ),
  pool: Ecto.Adapters.SQL.Sandbox

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :simple_budget, SimpleBudgetWeb.Endpoint,
  http: [port: 4002],
  server: false

# Print only warnings and errors during test
config :logger, level: :warn

config :simple_budget,
  token_key: "development-use-only"
