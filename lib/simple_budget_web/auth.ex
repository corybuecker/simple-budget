defmodule SimpleBudgetWeb.Auth do
  @moduledoc false
  import Plug.Conn
  import Phoenix.Controller, only: [redirect: 2]

  def check_google_session(conn, _) do
    case Application.get_env(:simple_budget, :skip_auth) do
      true ->
        conn

      _ ->
        case conn.path_info do
          ["login"] -> conn
          _ -> redirect_without_token(conn)
        end
    end
  end

  def check_google_session_api(conn, _) do
    case Application.get_env(:simple_budget, :skip_auth) do
      true ->
        conn

      _ ->
        case get_session(conn, :token) do
          token when byte_size(token) > 0 ->
            conn

          _ ->
            conn
            |> send_resp(401, "{}")
            |> halt()
        end
    end
  end

  defp redirect_without_token(conn) do
    case get_session(conn, :token) do
      token when byte_size(token) > 0 ->
        conn

      _ ->
        conn |> redirect(to: "/login") |> halt()
    end
  end
end
