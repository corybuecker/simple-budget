defmodule SimpleBudgetWeb.SavingView do
  use SimpleBudgetWeb, :view
  alias SimpleBudgetWeb.SavingView

  def render("index.json", %{savings: savings}) do
    %{data: render_many(savings, SavingView, "saving.json")}
  end

  def render("show.json", %{saving: saving}) do
    %{data: render_one(saving, SavingView, "saving.json")}
  end

  def render("saving.json", %{saving: saving}) do
    %{
      id: saving.id,
      title: saving.title,
      amount: Decimal.to_float(saving.amount)
    }
  end
end
