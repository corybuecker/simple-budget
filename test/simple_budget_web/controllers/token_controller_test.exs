defmodule SimpleBudgetWeb.TokenControllerTest do
  use SimpleBudgetWeb.ConnCase

  test "create token with dummy configuration", %{conn: conn} do
    Application.put_env(:simple_budget, :authentication, :dummy)

    conn =
      put_req_header(conn, "content-type", "application/json")
      |> post(Routes.token_path(conn, :create), email: "test@user.com")

    response = json_response(conn, 200)
    assert Map.has_key?(response, "idtoken")
  end

  test "create token with non-dummy configuration", %{conn: conn} do
    Application.put_env(:simple_budget, :authentication, :google)

    conn =
      put_req_header(conn, "content-type", "application/json")
      |> post(Routes.token_path(conn, :create), email: "test@user.com")

    Application.put_env(:simple_budget, :authentication, :dummy)
    assert json_response(conn, 401)
  end
end
