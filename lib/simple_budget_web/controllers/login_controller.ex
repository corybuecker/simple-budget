defmodule SimpleBudgetWeb.LoginController do
  use SimpleBudgetWeb, :controller

  require Logger

  alias SimpleBudget.TokenAuth.Dummy
  alias SimpleBudget.TokenAuth.Google

  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    render(conn, "index.html")
  end

  # def create(conn, %{"idtoken" => token}) do
  #   case authentication_method().(token) do
  #     {:ok, _} ->
  #       conn
  #       |> put_session(:token, token)
  #       |> send_resp(:created, "")

  #     {:error, error} ->
  #       Logger.error(error)
  #       conn |> send_resp(:unauthorized, "")
  #   end
  # end

  def create(conn, %{"username" => username}) do
    conn
    |> put_session(:token, username)
    |> send_resp(:no_content, "")
  end

  defp authentication_method do
    case Application.get_env(:simple_budget, :authentication) do
      :dummy ->
        fn t -> Dummy.verify_and_validate_token(t) end

      _ ->
        fn t -> Google.verify_and_validate_token(t) end
    end
  end
end
