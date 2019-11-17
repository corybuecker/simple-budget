defmodule SimpleBudgetWeb.Auth do
  import Plug.Conn
  import Phoenix.Controller, only: [redirect: 2]
  def init(_), do: nil

  def authenticated_session(conn, _) do
    case get_session(conn, :user_id) do
      user_id when is_integer(user_id) and user_id > 0 ->
        user = SimpleBudget.Users.get_user(user_id)
        conn = conn |> assign(:user, user)

        conn

      _ ->
        conn |> redirect(to: "/auth/login") |> halt()
    end
  end

  def authenticated_api_session(conn, _) do
    case get_session(conn, :user_id) do
      user_id when is_integer(user_id) and user_id > 0 ->
        user = SimpleBudget.Users.get_user(user_id)
        conn = conn |> assign(:user, user)

        conn

      _ ->
        conn
        |> put_resp_header("content-type", "application/json")
        |> send_resp(401, "{}")
        |> halt()
    end
  end
end
