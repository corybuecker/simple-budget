defmodule SimpleBudgetWeb.CalculationsController do
  use SimpleBudgetWeb, :controller

  alias SimpleBudget.Calculations.Daily

  action_fallback(SimpleBudgetWeb.FallbackController)

  def index(conn, _params) do
    render(conn, "index.json", daily: Daily.all())
  end
end
