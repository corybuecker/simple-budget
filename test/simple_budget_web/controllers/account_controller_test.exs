defmodule SimpleBudgetWeb.AccountControllerTest do
  use SimpleBudgetWeb.ConnCase

  alias SimpleBudget.Accounts
  alias SimpleBudget.Accounts.Account

  @create_attrs %{name: "some name", balance: 123, debt: false}
  @update_attrs %{name: "some updated name", balance: 345, debt: true}
  @invalid_attrs %{name: nil}

  def fixture(:account) do
    {:ok, account} = Accounts.create_account(@create_attrs)
    account
  end

  setup %{conn: conn} do
    {:ok,
     conn:
       conn
       |> init_test_session(%{token: "validid"})
       |> put_req_header("accept", "application/json")}
  end

  describe "show" do
    test "missing account", %{conn: conn} do
      conn = get(conn, Routes.account_path(conn, :show, -1))
      assert json_response(conn, 404) =~ "Not Found"
    end
  end

  describe "index" do
    test "lists all accounts", %{conn: conn} do
      conn = get(conn, Routes.account_path(conn, :index))
      assert json_response(conn, 200)["data"] == []
    end

    test "includes adjusments for each account", %{conn: conn} do
      account = fixture(:account)
      Accounts.create_adjustment(%{account: account, total: 500.0})
      conn = get(conn, Routes.account_path(conn, :index))

      assert json_response(conn, 200)["data"] == [
               %{
                 "balance" => 123.0,
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
      conn = post(conn, Routes.account_path(conn, :create), account: @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.account_path(conn, :create), account: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update account" do
    setup [:create_account]

    test "renders account when data is valid", %{conn: conn, account: %Account{id: id} = account} do
      conn = put(conn, Routes.account_path(conn, :update, account), account: @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn, account: account} do
      conn = put(conn, Routes.account_path(conn, :update, account), account: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete account" do
    setup [:create_account]

    test "deletes chosen account", %{conn: conn, account: account} do
      conn = delete(conn, Routes.account_path(conn, :delete, account))
      assert response(conn, 204)
    end
  end

  defp create_account(_) do
    account = fixture(:account)
    {:ok, account: account}
  end
end
