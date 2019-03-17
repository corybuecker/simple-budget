defmodule SimpleBudgetWeb.LoginControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, "/auth/login")
    assert html_response(conn, 200) =~ "new Login"
  end
end
