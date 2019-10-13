defmodule SimpleBudgetWeb.TokenView do
  use SimpleBudgetWeb, :view

  def render("create.json", %{token: token}) do
    %{token: token}
  end
end
