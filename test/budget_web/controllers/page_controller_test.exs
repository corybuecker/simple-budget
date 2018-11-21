defmodule SimpleBudgetWeb.PageControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn |> init_test_session(%{token: "validid"}), "/")
    assert html_response(conn, 200) =~ "app\.js"
  end

  test "unauthorized GET /", %{conn: conn} do
    conn = get(conn, "/")
    assert redirected_to(conn, 302) =~ "/login"
  end

  test "not found", %{conn: conn} do
    assert_raise Phoenix.Router.NoRouteError, fn ->
      get(conn, "/unknown")
    end
  end
end
