defmodule SimpleBudgetWeb.LoginController do
  use SimpleBudgetWeb, :controller
  alias SimpleBudget.TokenAuth.Google
  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    render(conn, "index.html")
  end

  def create(conn, %{"idtoken" => token}) do
    case Google.verify_and_validate(token) do
      {:ok, _} ->
        conn
        |> put_session(:token, token)
        |> send_resp(:created, "")

      _ ->
        conn |> send_resp(:unauthorized, "")
    end
  end
end
