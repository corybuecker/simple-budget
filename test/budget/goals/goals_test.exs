defmodule SimpleBudget.GoalsTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Goals

  describe "goals" do
    alias SimpleBudget.Goals.Goal

    @valid_attrs %{
      end_date: ~D[2010-04-17],
      start_date: ~D[2010-04-17],
      target: 120.54,
      title: "some title"
    }
    @update_attrs %{
      end_date: ~D[2011-05-18],
      start_date: ~D[2011-05-18],
      target: 456.72,
      title: "some updated title"
    }
    @invalid_attrs %{end_date: nil, start_date: nil, target: nil, title: nil}

    def goal_fixture(attrs \\ %{}) do
      {:ok, goal} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Goals.create_goal()

      goal
    end

    test "list_goals/0 returns all goals" do
      goal = goal_fixture()
      assert Goals.list_goals() == [goal]
    end

    test "get_goal!/1 returns the goal with given id" do
      goal = goal_fixture()
      assert Goals.get_goal!(goal.id) == goal
    end

    test "create_goal/1 with valid data creates a goal" do
      assert {:ok, %Goal{} = goal} = Goals.create_goal(@valid_attrs)
      assert goal.end_date == ~D[2010-04-17]
      assert goal.start_date == ~D[2010-04-17]
      assert goal.target == Decimal.new(120.54)
      assert goal.title == "some title"
    end

    test "create_goal/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Goals.create_goal(@invalid_attrs)
    end

    test "update_goal/2 with valid data updates the goal" do
      goal = goal_fixture()
      assert {:ok, goal} = Goals.update_goal(goal, @update_attrs)
      assert %Goal{} = goal
      assert goal.end_date == ~D[2011-05-18]
      assert goal.start_date == ~D[2011-05-18]
      assert goal.target == Decimal.new(456.72)
      assert goal.title == "some updated title"
    end

    test "update_goal/2 with invalid data returns error changeset" do
      goal = goal_fixture()
      assert {:error, %Ecto.Changeset{}} = Goals.update_goal(goal, @invalid_attrs)
      assert goal == Goals.get_goal!(goal.id)
    end

    test "delete_goal/1 deletes the goal" do
      goal = goal_fixture()
      assert {:ok, %Goal{}} = Goals.delete_goal(goal)
      assert_raise Ecto.NoResultsError, fn -> Goals.get_goal!(goal.id) end
    end

    test "change_goal/1 returns a goal changeset" do
      goal = goal_fixture()
      assert %Ecto.Changeset{} = Goals.change_goal(goal)
    end
  end
end
