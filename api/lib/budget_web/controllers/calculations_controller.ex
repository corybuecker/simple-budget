defmodule BudgetWeb.CalculationsController do
  use BudgetWeb, :controller

  alias Budget.Calculations.Daily

  action_fallback(BudgetWeb.FallbackController)

  def index(conn, _params) do
    render(conn, "index.json", daily: Daily.all())
  end
end
