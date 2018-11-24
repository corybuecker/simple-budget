defmodule SimpleBudget.Calculations.Daily do
  @moduledoc false
  import Ecto.Query

  alias SimpleBudget.Repo

  def all do
    remaining = remaining()
    remaining_per_day = remaining_per_day(remaining)

    %{
      remaining: remaining,
      remaining_per_day: remaining_per_day
    }
  end

  defp remaining do
    remaining_credit_after_debt()
    |> Decimal.sub(savings())
    |> Decimal.sub(goals())
  end

  defp remaining_credit_after_debt do
    Decimal.sub(credits(), debts())
  end

  defp remaining_per_day(remaining) do
    days_left =
      Timex.now()
      |> Timex.end_of_month()
      |> Timex.diff(Timex.now(), :days)

    if days_left == 0 do
      remaining
    else
      Decimal.div(remaining, Decimal.new(days_left))
    end
  end

  defp credits do
    credits_query =
      from(
        a in "accounts",
        where: a.debt == false,
        select: sum(a.balance)
      )

    credits = credits_query |> Repo.one()

    adjustments_query =
      from(
        a in "accounts",
        where: a.debt == false,
        join: adjustments in "adjustments",
        on: adjustments.account_id == a.id,
        select: sum(adjustments.total)
      )

    adjustments = adjustments_query |> Repo.one()

    Decimal.add(credits || Decimal.new(0), adjustments || Decimal.new(0))
  end

  defp debts do
    debts_query =
      from(
        a in "accounts",
        where: a.debt == true,
        select: sum(a.balance)
      )

    debts = debts_query |> Repo.one()

    adjustments_query =
      from(
        a in "accounts",
        where: a.debt == true,
        join: adjustments in "adjustments",
        on: adjustments.account_id == a.id,
        select: sum(adjustments.total)
      )

    adjustments = adjustments_query |> Repo.one()

    Decimal.add(debts || Decimal.new(0), adjustments || Decimal.new(0))
  end

  defp savings do
    savings_query =
      from(
        a in "savings",
        select: sum(a.amount)
      )

    savings = savings_query |> Repo.one()

    Decimal.new(savings || Decimal.new(0))
  end

  defp goals do
    goals_query =
      from(
        a in "goals",
        select:
          sum(
            fragment("(target / (end_date - start_date)) * DATE_PART('day', now() - start_date)")
          )
      )

    goals =
      goals_query
      |> Repo.one()

    Decimal.new(goals || Decimal.new(0))
  end
end
