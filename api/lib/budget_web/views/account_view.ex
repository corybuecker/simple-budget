defmodule BudgetWeb.AccountView do
  use BudgetWeb, :view
  alias BudgetWeb.AccountView
  alias BudgetWeb.AdjustmentView

  def render("index.json", %{accounts: accounts}) do
    %{data: render_many(accounts, AccountView, "account_with_adjustments.json")}
  end

  def render("show.json", %{account: account}) do
    %{data: render_one(account, AccountView, "account.json")}
  end

  def render("account.json", %{account: account}) do
    %{id: account.id, name: account.name, balance: account.balance, debt: account.debt}
  end

  def render("account_with_adjustments.json", %{account: account}) do
    account
    |> Map.take([:id, :name, :balance, :debt])
    |> Map.put(
         :adjustments,
         render_many(account.adjustments, AdjustmentView, "adjustment.json")
       )
  end
end
