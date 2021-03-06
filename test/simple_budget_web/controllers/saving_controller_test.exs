defmodule SimpleBudgetWeb.SavingControllerTest do
  use SimpleBudgetWeb.ConnCase

  alias SimpleBudget.Savings
  alias SimpleBudget.Savings.Saving

  @create_attrs %{amount: 120.5, title: "some title"}
  @update_attrs %{amount: 456.7, title: "some updated title"}
  @invalid_attrs %{amount: nil, title: nil}

  def fixture(:saving) do
    {:ok, saving} = Savings.create_saving(@create_attrs)
    saving
  end

  setup %{conn: conn} do
    {:ok,
     conn:
       conn
       |> init_test_session(%{token: "validid"})
       |> put_req_header("accept", "application/json")}
  end

  describe "index" do
    test "lists all savings", %{conn: conn} do
      conn = get(conn, Routes.saving_path(conn, :index))
      assert json_response(conn, 200)["data"] == []
    end
  end

  describe "create saving" do
    test "renders saving when data is valid", %{conn: conn} do
      conn = post(conn, Routes.saving_path(conn, :create), saving: @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.saving_path(conn, :create), saving: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update saving" do
    setup [:create_saving]

    test "renders saving when data is valid", %{conn: conn, saving: %Saving{id: id} = saving} do
      conn = put(conn, Routes.saving_path(conn, :update, saving), saving: @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn, saving: saving} do
      conn = put(conn, Routes.saving_path(conn, :update, saving), saving: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete saving" do
    setup [:create_saving]

    test "deletes chosen saving", %{conn: conn, saving: saving} do
      conn = delete(conn, Routes.saving_path(conn, :delete, saving))
      assert response(conn, 204)
    end
  end

  defp create_saving(_) do
    saving = fixture(:saving)
    {:ok, saving: saving}
  end
end
