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
    case authentication_method().(token) do
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

  defp authentication_method do
    case Application.get_env(:simple_budget, :authentication) do
      :email ->
        fn t -> Email.verify_and_validate_token(t) end

      _ ->
        fn t -> Google.verify_and_validate_token(t) end
    end
  end
end
