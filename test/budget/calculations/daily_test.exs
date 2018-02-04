defmodule Budget.Calculations.DailyTest do
  use Budget.DataCase

  alias Budget.Calculations.Daily
  alias Budget.Accounts
  alias Budget.Savings
  alias Budget.Goals

  setup do
    {:ok, credit} = Accounts.create_account(%{name: "some name", balance: 750, debt: false})
    {:ok, _debt} = Accounts.create_account(%{name: "some name", balance: 5, debt: true})

    {:ok, _adjustment} =
      Accounts.create_adjustment(%{account_id: credit.id, title: "adjustment", total: -50.0})

    {:ok, _adjustment} =
      Accounts.create_adjustment(%{account_id: credit.id, title: "adjustment", total: -3.0})

    {:ok, _saving} = Savings.create_saving(%{title: "savings", amount: 30.0})

    {:ok, _goal} =
      Goals.create_goal(%{
        title: "goal",
        start_date: Timex.shift(Timex.today(), days: -1),
        end_date: Timex.shift(Timex.today(), days: 1),
        target: 100
      })

    :ok
  end

  test "calculates the remaining amount per day" do
    # not a great test since it duplicates the implementation, TODO: find a
    # way to freeze Timex to a given datetime.
    days_left =
      cond do
        Timex.today() == Timex.today() |> Timex.end_of_month() ->
          1

        true ->
          Timex.Interval.new(from: Timex.today(), until: Timex.today() |> Timex.end_of_month())
          |> Timex.Interval.duration(:days)
      end

    assert Daily.all() == %{
             remaining: Decimal.new("612.000"),
             remaining_per_day: Decimal.div(Decimal.new("612.000"), Decimal.new(days_left))
           }
  end
end
