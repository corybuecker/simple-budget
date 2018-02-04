defmodule Budget.Calculations.Daily do
  import Ecto.Query

  alias Budget.Repo
  alias Timex.Interval

  def all do
    remaining = remaining()
    remaining_per_day = remaining_per_day(remaining)

    %{
      remaining: remaining,
      remaining_per_day: remaining_per_day
    }
  end

  defp remaining do
    credits() - debts() - savings() - goals()
  end

  defp remaining_per_day(remaining) do
    days_left =
      Interval.new(from: Timex.today(), until: Timex.today() |> Timex.end_of_month())
      |> Interval.duration(:days)

    cond do
      days_left == 0 -> remaining
      true -> remaining / days_left
    end
  end

  defp credits do
    credits =
      from(
        a in "accounts",
        where: a.debt == false,
        select: sum(a.balance)
      )
      |> Repo.one()

    adjustments =
      from(
        a in "accounts",
        where: a.debt == false,
        join: adjustments in "adjustments",
        on: adjustments.account_id == a.id,
        select: sum(adjustments.total)
      )
      |> Repo.one()

    (credits || 0) + (adjustments || 0)
  end

  defp debts do
    debts =
      from(
        a in "accounts",
        where: a.debt == true,
        select: sum(a.balance)
      )
      |> Repo.one()

    adjustments =
      from(
        a in "accounts",
        where: a.debt == true,
        join: adjustments in "adjustments",
        on: adjustments.account_id == a.id,
        select: sum(adjustments.total)
      )
      |> Repo.one()

    (debts || 0) + (adjustments || 0)
  end

  defp savings do
    savings =
      from(
        a in "savings",
        select: sum(a.amount)
      )
      |> Repo.one()

    savings || 0
  end

  defp goals do
    goals =
      from(
        a in "goals",
        select:
          sum(
            fragment("(target / (end_date - start_date)) * DATE_PART('day', now() - start_date)")
          )
      )
      |> Repo.one()

    goals || 0
  end
end
