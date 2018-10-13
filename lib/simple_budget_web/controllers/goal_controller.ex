defmodule SimpleBudgetWeb.GoalController do
  use SimpleBudgetWeb, :controller

  alias SimpleBudget.Goals
  alias SimpleBudget.Goals.Goal

  action_fallback SimpleBudgetWeb.FallbackController

  def index(conn, _params) do
    goals = Goals.list_goals()
    render(conn, "index.json", goals: goals)
  end

  def create(conn, %{"goal" => goal_params}) do
    with {:ok, %Goal{} = goal} <- Goals.create_goal(goal_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.goal_path(conn, :show, goal))
      |> render("show.json", goal: goal)
    end
  end

  def show(conn, %{"id" => id}) do
    goal = Goals.get_goal!(id)
    render(conn, "show.json", goal: goal)
  end

  def update(conn, %{"id" => id, "goal" => goal_params}) do
    goal = Goals.get_goal!(id)

    with {:ok, %Goal{} = goal} <- Goals.update_goal(goal, goal_params) do
      render(conn, "show.json", goal: goal)
    end
  end

  def delete(conn, %{"id" => id}) do
    goal = Goals.get_goal!(id)

    with {:ok, %Goal{}} <- Goals.delete_goal(goal) do
      send_resp(conn, :no_content, "")
    end
  end
end
