defmodule SimpleBudgetWeb.PageControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, "/")
    assert html_response(conn, 200) =~ "main\.js"
  end

  test "GET /healthcheck", %{conn: conn} do
    conn = get(conn, "/healthcheck")
    assert response(conn, 200) =~ ""
  end
end
