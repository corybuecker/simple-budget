defmodule SimpleBudgetWeb.AccountView do
  use SimpleBudgetWeb, :view
  alias SimpleBudgetWeb.AccountView
  alias SimpleBudgetWeb.AdjustmentView

  def render("index.json", %{accounts: accounts}) do
    %{data: render_many(accounts, AccountView, "account_with_adjustments.json")}
  end

  def render("show.json", %{account: account}) do
    %{data: render_one(account, AccountView, "account.json")}
  end

  def render("update.json", %{account: account}) do
    %{data: render_one(account, AccountView, "account_with_adjustments.json")}
  end

  def render("account.json", %{account: account}) do
    account |> account_to_map()
  end

  def render("account_with_adjustments.json", %{account: account}) do
    account
    |> account_to_map()
    |> Map.put(
      :adjustments,
      render_many(account.adjustments, AdjustmentView, "adjustment.json")
    )
  end

  defp account_to_map(account) do
    %{
      id: account.id,
      name: account.name,
      balance: Decimal.to_float(account.balance),
      debt: account.debt
    }
  end
end
