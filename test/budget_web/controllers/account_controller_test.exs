defmodule BudgetWeb.AccountControllerTest do
  use BudgetWeb.ConnCase

  alias Budget.Accounts
  alias Budget.Accounts.Account

  @create_attrs %{name: "some name", balance: 123, debt: false}
  @update_attrs %{name: "some updated name", balance: 345, debt: true}
  @invalid_attrs %{name: nil}

  def fixture(:account) do
    {:ok, account} = Accounts.create_account(@create_attrs)
    account
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all accounts", %{conn: conn} do
      conn = get(conn, account_path(conn, :index))
      assert json_response(conn, 200)["data"] == []
    end

    test "includes adjusments for each account", %{conn: conn} do
      account = fixture(:account)
      Accounts.create_adjustment(%{account: account, total: 500.0})
      conn = get(conn, account_path(conn, :index))

      assert json_response(conn, 200)["data"] == [
               %{
                 "balance" => "123.00",
                 "id" => account.id,
                 "name" => "some name",
                 "debt" => false,
                 "adjustments" => []
               }
             ]
    end
  end

  describe "create account" do
    test "renders account when data is valid", %{conn: conn} do
      conn = post(conn, account_path(conn, :create), account: @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)["data"]

      conn = get(conn, account_path(conn, :show, id))

      assert json_response(conn, 200)["data"] == %{
               "id" => id,
               "name" => "some name",
               "balance" => "123.00",
               "debt" => false
             }
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, account_path(conn, :create), account: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update account" do
    setup [:create_account]

    test "renders account when data is valid", %{conn: conn, account: %Account{id: id} = account} do
      conn = put(conn, account_path(conn, :update, account), account: @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)["data"]

      conn = get(conn, account_path(conn, :show, id))

      assert json_response(conn, 200)["data"] == %{
               "id" => id,
               "name" => "some updated name",
               "balance" => "345.00",
               "debt" => true
             }
    end

    test "renders errors when data is invalid", %{conn: conn, account: account} do
      conn = put(conn, account_path(conn, :update, account), account: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete account" do
    setup [:create_account]

    test "deletes chosen account", %{conn: conn, account: account} do
      conn = delete(conn, account_path(conn, :delete, account))
      assert response(conn, 204)

      assert_error_sent(404, fn ->
        get(conn, account_path(conn, :show, account))
      end)
    end
  end

  defp create_account(_) do
    account = fixture(:account)
    {:ok, account: account}
  end
end
