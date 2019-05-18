defmodule SimpleBudgetWeb.AdjustmentControllerTest do
  use SimpleBudgetWeb.ConnCase

  alias SimpleBudget.Accounts
  alias SimpleBudget.Accounts.Adjustment

  @account_id 42
  @create_attrs %{total: 120.5, title: "test"}
  @update_attrs %{total: 456.7}
  @invalid_attrs %{total: nil}

  def fixture(:adjustment) do
    {:ok, adjustment} =
      Accounts.create_adjustment(@create_attrs |> Map.merge(%{account_id: @account_id}))

    adjustment
  end

  def fixture(:account) do
    {:ok, account} =
      Accounts.create_account(%{id: @account_id, name: "test", balance: 242, debt: false})

    account
  end

  setup %{conn: conn} do
    {:ok,
     conn:
       conn
       |> init_test_session(%{token: "validid"})
       |> put_req_header("accept", "application/json"),
     account: fixture(:account)}
  end

  describe "create adjustments" do
    test "renders adjustments when data is valid", %{conn: conn, account: account} do
      conn =
        post(
          conn,
          Routes.account_adjustment_path(conn, :create, account),
          adjustment: @create_attrs
        )

      assert %{"id" => id} = json_response(conn, 201)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn, account: account} do
      conn =
        post(
          conn,
          Routes.account_adjustment_path(conn, :create, account),
          adjustment: @invalid_attrs
        )

      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update adjustments" do
    setup [:create_adjustment]

    test "renders adjustments when data is valid", %{
      conn: conn,
      adjustment: %Adjustment{id: id} = adjustment,
      account: account
    } do
      conn =
        put(
          conn,
          Routes.account_adjustment_path(conn, :update, account, adjustment),
          adjustment: @update_attrs
        )

      assert %{"id" => ^id} = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{
      conn: conn,
      adjustment: adjustment,
      account: account
    } do
      conn =
        put(
          conn,
          Routes.account_adjustment_path(conn, :update, account, adjustment),
          adjustment: @invalid_attrs
        )

      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete adjustments" do
    setup [:create_adjustment]

    test "deletes chosen adjustments", %{conn: conn, adjustment: adjustment, account: account} do
      conn = delete(conn, Routes.account_adjustment_path(conn, :delete, account, adjustment))
      assert response(conn, 204)
    end
  end

  defp create_adjustment(_) do
    adjustment = fixture(:adjustment)
    {:ok, adjustment: adjustment}
  end
end
