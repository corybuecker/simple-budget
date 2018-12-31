defmodule SimpleBudgetWeb.PageControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn |> init_test_session(%{token: "validid"}), "/")
    assert html_response(conn, 200) =~ "app\.js"
  end

  test "GET /accounts", %{conn: conn} do
    conn = get(conn |> init_test_session(%{token: "validid"}), "/accounts")
    assert html_response(conn, 200) =~ "accounts\.js"
  end

  test "GET /goals", %{conn: conn} do
    conn = get(conn |> init_test_session(%{token: "validid"}), "/goals")
    assert html_response(conn, 200) =~ "goals\.js"
  end

  test "GET /savings", %{conn: conn} do
    conn = get(conn |> init_test_session(%{token: "validid"}), "/savings")
    assert html_response(conn, 200) =~ "savings\.js"
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
