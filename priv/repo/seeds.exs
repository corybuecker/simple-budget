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

%SimpleBudget.User{
  email: System.get_env("ADMIN_EMAIL"),
  identity: Ecto.UUID.generate(),
  preferences: %{}
}
|> SimpleBudget.Repo.insert!()
