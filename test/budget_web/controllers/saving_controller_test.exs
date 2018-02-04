defmodule BudgetWeb.SavingControllerTest do
  use BudgetWeb.ConnCase

  alias Budget.Savings
  alias Budget.Savings.Saving

  @create_attrs %{amount: 120.5, title: "some title"}
  @update_attrs %{amount: 456.7, title: "some updated title"}
  @invalid_attrs %{amount: nil, title: nil}

  def fixture(:saving) do
    {:ok, saving} = Savings.create_saving(@create_attrs)
    saving
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all savings", %{conn: conn} do
      conn = get conn, saving_path(conn, :index)
      assert json_response(conn, 200)["data"] == []
    end
  end

  describe "create saving" do
    test "renders saving when data is valid", %{conn: conn} do
      conn = post conn, saving_path(conn, :create), saving: @create_attrs
      assert %{"id" => id} = json_response(conn, 201)["data"]

      conn = get conn, saving_path(conn, :show, id)
      assert json_response(conn, 200)["data"] == %{
        "id" => id,
        "amount" => 120.5,
        "title" => "some title"}
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post conn, saving_path(conn, :create), saving: @invalid_attrs
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update saving" do
    setup [:create_saving]

    test "renders saving when data is valid", %{conn: conn, saving: %Saving{id: id} = saving} do
      conn = put conn, saving_path(conn, :update, saving), saving: @update_attrs
      assert %{"id" => ^id} = json_response(conn, 200)["data"]

      conn = get conn, saving_path(conn, :show, id)
      assert json_response(conn, 200)["data"] == %{
        "id" => id,
        "amount" => 456.7,
        "title" => "some updated title"}
    end

    test "renders errors when data is invalid", %{conn: conn, saving: saving} do
      conn = put conn, saving_path(conn, :update, saving), saving: @invalid_attrs
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete saving" do
    setup [:create_saving]

    test "deletes chosen saving", %{conn: conn, saving: saving} do
      conn = delete conn, saving_path(conn, :delete, saving)
      assert response(conn, 204)
      assert_error_sent 404, fn ->
        get conn, saving_path(conn, :show, saving)
      end
    end
  end

  defp create_saving(_) do
    saving = fixture(:saving)
    {:ok, saving: saving}
  end
end
