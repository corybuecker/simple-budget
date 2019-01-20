defmodule SimpleBudgetWeb.LoginControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, "/login")
    assert html_response(conn, 200) =~ "new Login"
  end
end
