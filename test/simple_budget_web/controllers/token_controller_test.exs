defmodule SimpleBudgetWeb.TokenControllerTest do
  use SimpleBudgetWeb.ConnCase
  alias SimpleBudget.Users

  def user_fixture(attrs \\ %{}) do
    {:ok, user} = Users.create_user(attrs)

    user
  end

  test "create token with email configuration", %{conn: conn} do
    Application.put_env(:simple_budget, :authentication, :email)
    user_fixture(%{email: "test@user.com", password: Argon2.hash_pwd_salt("password")})

    conn =
      conn
      |> put_req_header("content-type", "application/json")
      |> post(Routes.token_path(conn, :create), email: "test@user.com", password: "password")

    response = json_response(conn, 200)
    assert Map.has_key?(response, "localtoken")
  end

  test "create token with email configuration but missing user", %{conn: conn} do
    Application.put_env(:simple_budget, :authentication, :email)
    user_fixture(%{email: "test@user.com", password: Argon2.hash_pwd_salt("password")})

    conn =
      conn
      |> put_req_header("content-type", "application/json")
      |> post(Routes.token_path(conn, :create), email: "test@users.com", password: "password")

    assert json_response(conn, 401)
  end

  test "create token with non-email configuration", %{conn: conn} do
    Application.put_env(:simple_budget, :authentication, :google)

    conn =
      conn
      |> put_req_header("content-type", "application/json")
      |> post(Routes.token_path(conn, :create), email: "test@user.com")

    Application.put_env(:simple_budget, :authentication, :email)
    assert json_response(conn, 401)
  end
end
