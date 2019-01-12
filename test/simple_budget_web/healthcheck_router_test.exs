defmodule SimpleBudgetWeb.HealthcheckControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /healthcheck", %{conn: conn} do
    conn = get(conn, "/healthcheck")
    assert response(conn, 200)
  end
end
