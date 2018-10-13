defmodule SimpleBudget.Repo do
  use Ecto.Repo,
    otp_app: :simple_budget,
    adapter: Ecto.Adapters.Postgres
end
