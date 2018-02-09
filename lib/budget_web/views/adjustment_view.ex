defmodule BudgetWeb.AdjustmentView do
  use BudgetWeb, :view
  alias BudgetWeb.AdjustmentView

  def render("index.json", %{adjustments: adjustments}) do
    %{data: render_many(adjustments, AdjustmentView, "adjustment.json")}
  end

  def render("show.json", %{adjustment: adjustment}) do
    %{data: render_one(adjustment, AdjustmentView, "adjustment.json")}
  end

  def render("adjustment.json", %{adjustment: adjustment}) do
    %{
      id: adjustment.id,
      account_id: adjustment.account_id,
      title: adjustment.title,
      total: adjustment.total,
      total_cents: adjustment.total * 100.0
    }
  end
end
