# Script for populating the database. You can run it as:
#
#     mix run priv/repo/seeds.exs
#
# Inside the script, you can read and write to any of your
# repositories directly:
#
#     Budget.Repo.insert!(%Budget.SomeSchema{})
#
# We recommend using the bang functions (`insert!`, `update!`
# and so on) as they will fail if something goes wrong.

credit_card =
  Budget.Repo.insert!(%Budget.Accounts.Account{
    name: "Credit Card",
    balance: 1000.0,
    debt: true
  })

checking =
  Budget.Repo.insert!(%Budget.Accounts.Account{name: "Checking", balance: 1000.0, debt: false})

Budget.Repo.insert!(%Budget.Accounts.Adjustment{
  account_id: checking.id,
  total: -124.54,
  title: "Birthday Present"
})

Budget.Accounts.update_account(credit_card, %{balance: 845.24})

Budget.Repo.insert!(%Budget.Goals.Goal{
  title: "New Laptop",
  target: 2352.52,
  start_date: Timex.shift(Timex.today(), months: -1),
  end_date: Timex.shift(Timex.today(), months: 1)
})

Budget.Repo.insert!(%Budget.Savings.Saving{
  title: "Checking Account Buffer",
  amount: 250
})
