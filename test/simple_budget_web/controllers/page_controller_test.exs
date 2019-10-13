defmodule SimpleBudgetWeb.PageControllerTest do
  use SimpleBudgetWeb.ConnCase
  use Plug.Test

  test "GET /", %{conn: conn} do
    conn = get(conn, "/")
    assert html_response(conn, 200) =~ "app.js"
  end
end
