defmodule SimpleBudget.Calculations.DailyTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Accounts
  alias SimpleBudget.Users
  alias SimpleBudget.Calculations.Daily
  alias SimpleBudget.Goals
  alias SimpleBudget.Savings

  setup do
    user =
      with {:ok, user} <- Users.create_user(%{email: "test@test.com", password: "Test"}) do
        Ecto.Changeset.change(user)
      end

    {:ok, credit} =
      Accounts.create_account(%{name: "some name", balance: 750, debt: false, user: user})

    {:ok, _debt} =
      Accounts.create_account(%{name: "some name", balance: 5, debt: true, user: user})

    {:ok, _adjustment} =
      Accounts.create_adjustment(%{account_id: credit.id, title: "adjustment", total: -50.0})

    {:ok, _adjustment} =
      Accounts.create_adjustment(%{account_id: credit.id, title: "adjustment", total: -3.0})

    {:ok, _saving} = Savings.create_saving(%{title: "savings", amount: 30.0, user: user})

    {:ok, _goal} =
      Goals.create_goal(%{
        title: "goal",
        start_date: Timex.shift(Timex.today(), days: -1),
        end_date: Timex.shift(Timex.today(), days: 1),
        target: 100,
        user: user
      })

    :ok
  end

  test "calculates the remaining amount per day" do
    # not a great test since it duplicates the implementation, TODO: find a
    # way to freeze Timex to a given datetime.
    days_left =
      if Timex.today() == Timex.today() |> Timex.end_of_month() do
        1
      else
        Timex.Interval.new(from: Timex.today(), until: Timex.today() |> Timex.end_of_month())
        |> Timex.Interval.duration(:days)
      end

    assert Daily.all() ==
             %{
               remaining: Decimal.from_float(612.0),
               remaining_per_day: Decimal.div(Decimal.from_float(612.0), days_left)
             }
  end
end
