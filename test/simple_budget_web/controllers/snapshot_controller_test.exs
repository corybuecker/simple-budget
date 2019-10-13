defmodule SimpleBudgetWeb.SnapshotControllerTest do
  use SimpleBudgetWeb.ConnCase
  use Plug.Test
  alias SimpleBudget.Accounts

  @account_id 42

  def fixture(:account) do
    {:ok, account} =
      Accounts.create_account(%{id: @account_id, name: "test", balance: 1000, debt: false})

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

  describe "index" do
    test "lists all snapshots", %{conn: conn, account: account} do
      conn = get(conn, Routes.account_snapshot_path(conn, :index, account))
      assert json_response(conn, 200)["data"] == []
    end
  end

  describe "create snapshot" do
    test "creates a snapshot when the account is update", %{conn: conn, account: account} do
      assert Enum.empty?(Accounts.list_snapshots())

      conn =
        patch(
          conn,
          Routes.account_path(conn, :update, account),
          account: %{balance: 500}
        )

      account_id = account.id
      name = account.name

      assert %{"name" => ^name, "id" => ^account_id, "balance" => 500.0} =
               json_response(conn, 200)["data"]

      snapshots = Accounts.list_snapshots()
      assert length(snapshots) == 1
      [snapshot | []] = snapshots
      assert {:ok, snapshot.after} == Decimal.parse("500.00")
    end

    test "renders errors when data is invalid", %{conn: conn, account: account} do
      conn =
        patch(
          conn,
          Routes.account_path(conn, :update, account),
          account: %{debt: nil}
        )

      assert json_response(conn, 422)["errors"] != %{}
    end
  end
end
