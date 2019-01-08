defmodule SimpleBudgetWeb.TokenView do
  use SimpleBudgetWeb, :view

  def render("create.json", %{token: token}) do
    %{idtoken: token}
  end
end
