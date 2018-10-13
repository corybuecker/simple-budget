defmodule SimpleBudgetWeb.SavingController do
  use SimpleBudgetWeb, :controller

  alias SimpleBudget.Savings
  alias SimpleBudget.Savings.Saving

  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    savings = Savings.list_savings()
    render(conn, "index.json", savings: savings)
  end

  def create(conn, %{"saving" => saving_params}) do
    with {:ok, %Saving{} = saving} <- Savings.create_saving(saving_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.saving_path(conn, :show, saving))
      |> render("show.json", saving: saving)
    end
  end

  def show(conn, %{"id" => id}) do
    saving = Savings.get_saving!(id)
    render(conn, "show.json", saving: saving)
  end

  def update(conn, %{"id" => id, "saving" => saving_params}) do
    saving = Savings.get_saving!(id)

    with {:ok, %Saving{} = saving} <- Savings.update_saving(saving, saving_params) do
      render(conn, "show.json", saving: saving)
    end
  end

  def delete(conn, %{"id" => id}) do
    saving = Savings.get_saving!(id)

    with {:ok, %Saving{}} <- Savings.delete_saving(saving) do
      send_resp(conn, :no_content, "")
    end
  end
end
