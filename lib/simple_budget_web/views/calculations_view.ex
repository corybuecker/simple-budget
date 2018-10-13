defmodule SimpleBudgetWeb.CalculationsView do
  use SimpleBudgetWeb, :view

  def render("index.json", %{daily: daily}) do
    %{data: daily}
  end
end
