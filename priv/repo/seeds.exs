# Script for populating the database. You can run it as:
#
#     mix run priv/repo/seeds.exs
#
# Inside the script, you can read and write to any of your
# repositories directly:
#
#     SimpleBudget.Repo.insert!(%SimpleBudget.SomeSchema{})
#
# We recommend using the bang functions (`insert!`, `update!`
# and so on) as they will fail if something goes wrong.
user =
  SimpleBudget.Repo.insert!(%SimpleBudget.Users.User{
    email: "test@user.com",
    password: Argon2.hash_pwd_salt("password")
  })

credit_card =
  SimpleBudget.Repo.insert!(%SimpleBudget.Accounts.Account{
    name: "Credit Card",
    balance: 1000.0,
    debt: true,
    user: user
  })

checking =
  SimpleBudget.Repo.insert!(%SimpleBudget.Accounts.Account{
    name: "Checking",
    balance: 1000.0,
    debt: false,
    user: user
  })

SimpleBudget.Repo.insert!(%SimpleBudget.Accounts.Adjustment{
  account_id: checking.id,
  total: -124.54,
  title: "Birthday Present"
})

SimpleBudget.Repo.insert!(%SimpleBudget.Accounts.Adjustment{
  account_id: checking.id,
  total: 230.12,
  title: "Work Reimbursement"
})

SimpleBudget.Accounts.update_account(credit_card, %{balance: 845.24})

SimpleBudget.Repo.insert!(%SimpleBudget.Goals.Goal{
  title: "New Laptop",
  target: 2352.52,
  start_date: Timex.shift(Timex.today(), months: -1),
  end_date: Timex.shift(Timex.today(), months: 1),
  user: user
})

SimpleBudget.Repo.insert!(%SimpleBudget.Savings.Saving{
  title: "Checking Account Buffer",
  amount: 250,
  user: user
})
