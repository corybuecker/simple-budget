defmodule SimpleBudgetWeb.LoginController do
  use SimpleBudgetWeb, :controller

  require Logger

  alias SimpleBudget.TokenAuth.Email

  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    conn
    |> put_resp_header("cache-control", "no-store, private")
    |> render("index.html")
  end

  def create(conn, %{"token" => token}) do
    case Email.verify_and_validate_token(token) do
      {:ok, _} ->
        conn
        |> fetch_session()
        |> put_session(:token, token)
        |> send_resp(:created, "")

      {:error, error} ->
        Logger.error(error)
        conn |> send_resp(:unauthorized, "")
    end
  end

  def logout(conn, _params) do
    conn
    |> clear_session()
    |> put_resp_header("cache-control", "no-store, private")
    |> render("delete.html")
  end
end
