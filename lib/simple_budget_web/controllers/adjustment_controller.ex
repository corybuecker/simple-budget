defmodule SimpleBudgetWeb.AdjustmentController do
  use SimpleBudgetWeb, :controller

  alias SimpleBudget.Accounts
  alias SimpleBudget.Accounts.Adjustment

  action_fallback(SimpleBudgetWeb.FallbackController)

  def create(conn, %{"adjustment" => adjustment_params, "account_id" => account_id}) do
    with {:ok, %Adjustment{} = adjustment} <-
           Accounts.create_adjustment(
             adjustment_params
             |> Map.merge(%{"account_id" => account_id})
           ) do
      conn
      |> put_status(:created)
      |> put_resp_header(
        "location",
        Routes.account_adjustment_path(conn, :show, account_id, adjustment)
      )
      |> render("show.json", adjustment: adjustment)
    end
  end

  def show(conn, %{"id" => id}) do
    adjustment = Accounts.get_adjustment!(id)
    render(conn, "show.json", adjustment: adjustment)
  end

  def update(conn, %{"id" => id, "adjustment" => adjustment_params}) do
    adjustment = Accounts.get_adjustment!(id)

    with {:ok, %Adjustment{} = adjustment} <-
           Accounts.update_adjustment(adjustment, adjustment_params) do
      render(conn, "show.json", adjustment: adjustment)
    end
  end

  def delete(conn, %{"id" => id}) do
    adjustment = Accounts.get_adjustment!(id)

    with {:ok, %Adjustment{}} <- Accounts.delete_adjustment(adjustment) do
      send_resp(conn, :no_content, "")
    end
  end
end
