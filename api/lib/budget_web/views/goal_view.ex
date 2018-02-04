defmodule BudgetWeb.GoalView do
  use BudgetWeb, :view
  alias BudgetWeb.GoalView

  def render("index.json", %{goals: goals}) do
    %{data: render_many(goals, GoalView, "goal.json")}
  end

  def render("show.json", %{goal: goal}) do
    %{data: render_one(goal, GoalView, "goal.json")}
  end

  def render("goal.json", %{goal: goal}) do
    %{
      id: goal.id,
      title: goal.title,
      start_date: goal.start_date,
      end_date: goal.end_date,
      target: goal.target
    }
  end
end
