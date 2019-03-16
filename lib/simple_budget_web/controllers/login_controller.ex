defmodule SimpleBudgetWeb.LoginController do
  use SimpleBudgetWeb, :controller

  require Logger

  alias SimpleBudget.TokenAuth.Email
  alias SimpleBudget.TokenAuth.Google

  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    render(conn, "index.html")
  end

  def create(conn, %{"idtoken" => token}) do
    case Google.verify_and_validate_token(token) do
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

  def create(conn, %{"localtoken" => token}) do
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
end
