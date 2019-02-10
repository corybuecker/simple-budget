defmodule SimpleBudgetWeb.Auth do
  @moduledoc false
  import Plug.Conn
  import Phoenix.Controller, only: [redirect: 2]

  def check_authenticated_session(conn, _) do
    case get_session(conn, :token) do
      token when byte_size(token) > 0 ->
        conn

      _ ->
        conn |> redirect(to: "/login") |> halt()
    end
  end

  def check_authenticated_api_session(conn, _) do
    case get_session(conn, :token) do
      token when byte_size(token) > 0 ->
        conn

      _ ->
        conn
        |> put_resp_header("content-type", "application/json")
        |> send_resp(401, "{}")
        |> halt()
    end
  end
end
