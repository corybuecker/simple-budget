defmodule SimpleBudget.Repo.Migrations.AddUsersTable do
  use Ecto.Migration

  def change do
    create table(:users) do
      add(:email, :string, null: false)

      timestamps()
    end
  end
end
