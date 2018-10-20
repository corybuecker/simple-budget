defmodule SimpleBudgetWeb.Auth do
  import Plug.Conn
  import Phoenix.Controller, only: [redirect: 2]

  def check_google_session(conn, _) do
    case conn.path_info do
      ["login"] -> conn
      _ -> redirect_without_token(conn)
    end
  end

  def check_google_session_api(conn, _) do
    case get_session(conn, :token) do
      token when byte_size(token) > 0 ->
        conn

      _ ->
        conn
        |> send_resp(401, "{}")
        |> halt()
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
