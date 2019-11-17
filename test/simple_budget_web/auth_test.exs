defmodule SimpleBudgetWeb.AuthTest do
  use ExUnit.Case, async: true
  use Plug.Test

  use SimpleBudget.DataCase

  alias SimpleBudgetWeb.Auth

  test "user not logged in redirects and halts the connection" do
    conn =
      conn(:get, "/")
      |> init_test_session(%{})
      |> Auth.authenticated_session(nil)

    assert conn.status == 302
    assert conn.halted
  end

  test "user logged assigns the user and does not halt the connection" do
    {:ok, user} = SimpleBudget.Users.create_user(%{email: "test@example.com", password: "test"})

    conn =
      conn(:get, "/")
      |> init_test_session(%{user_id: user.id})
      |> Auth.authenticated_session(nil)

    refute conn.halted
    assert conn.assigns[:user] == user
  end

  describe "api" do
    test "user not logged in redirects and halts the connection" do
      conn =
        conn(:get, "/")
        |> init_test_session(%{})
        |> Auth.authenticated_api_session(nil)

      assert conn.status == 401
      assert conn.halted
    end

    test "user logged assigns the user and does not halt the connection" do
      {:ok, user} = SimpleBudget.Users.create_user(%{email: "test@example.com", password: "test"})

      conn =
        conn(:get, "/")
        |> init_test_session(%{user_id: user.id})
        |> Auth.authenticated_api_session(nil)

      refute conn.halted
      assert conn.assigns[:user] == user
    end
  end
end
