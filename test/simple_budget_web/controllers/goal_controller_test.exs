defmodule SimpleBudgetWeb.GoalControllerTest do
  use SimpleBudgetWeb.ConnCase

  alias SimpleBudget.Goals
  alias SimpleBudget.Goals.Goal

  @create_attrs %{
    end_date: ~D[2010-04-17],
    start_date: ~D[2010-04-17],
    target: 120.5,
    title: "some title"
  }
  @update_attrs %{
    end_date: ~D[2011-05-18],
    start_date: ~D[2011-05-18],
    target: 456.7,
    title: "some updated title"
  }
  @invalid_attrs %{end_date: nil, start_date: nil, target: nil, title: nil}

  def fixture(:goal) do
    {:ok, goal} = Goals.create_goal(@create_attrs)
    goal
  end

  setup %{conn: conn} do
    {:ok,
     conn:
       conn
       |> init_test_session(%{token: "validid"})
       |> put_req_header("accept", "application/json")}
  end

  describe "index" do
    test "lists all goals", %{conn: conn} do
      conn = get(conn, Routes.goal_path(conn, :index))
      assert json_response(conn, 200)["data"] == []
    end
  end

  describe "create goal" do
    test "renders goal when data is valid", %{conn: conn} do
      conn = post(conn, Routes.goal_path(conn, :create), goal: @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.goal_path(conn, :create), goal: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update goal" do
    setup [:create_goal]

    test "renders goal when data is valid", %{conn: conn, goal: %Goal{id: id} = goal} do
      conn = put(conn, Routes.goal_path(conn, :update, goal), goal: @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn, goal: goal} do
      conn = put(conn, Routes.goal_path(conn, :update, goal), goal: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete goal" do
    setup [:create_goal]

    test "deletes chosen goal", %{conn: conn, goal: goal} do
      conn = delete(conn, Routes.goal_path(conn, :delete, goal))
      assert response(conn, 204)
    end
  end

  defp create_goal(_) do
    goal = fixture(:goal)
    {:ok, goal: goal}
  end
end
