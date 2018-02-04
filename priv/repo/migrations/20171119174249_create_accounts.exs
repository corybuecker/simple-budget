defmodule Budget.Repo.Migrations.CreateAccounts do
  use Ecto.Migration

  def change do
    create table(:accounts) do
      add(:name, :string)
      add(:balance, :float)
      add(:debt, :boolean)
      timestamps()
    end
  end
end
