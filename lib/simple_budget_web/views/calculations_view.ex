defmodule SimpleBudgetWeb.CalculationsView do
  use SimpleBudgetWeb, :view

  def render("index.json", %{daily: daily}) do
    %{
      data: %{
        remaining: Decimal.to_float(daily.remaining),
        remaining_per_day: Decimal.to_float(daily.remaining_per_day)
      }
    }
  end
end
