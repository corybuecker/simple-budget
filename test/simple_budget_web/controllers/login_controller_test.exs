defmodule SimpleBudgetWeb.LoginControllerTest do
  use SimpleBudgetWeb.ConnCase
  use Plug.Test

  test "GET /", %{conn: conn} do
    conn = get(conn, "/auth/login")
    assert html_response(conn, 200) =~ "login.js"
  end
end
