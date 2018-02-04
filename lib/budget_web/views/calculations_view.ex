defmodule BudgetWeb.CalculationsView do
  use BudgetWeb, :view

  def render("index.json", %{daily: daily}) do
    %{data: daily}
  end
end
