defmodule Budget.Repo.Migrations.CreateSnapshots do
  use Ecto.Migration

  def change do
    create table(:snapshots) do
      add :account_id, :integer
      add :before, :float
      add :after, :float

      timestamps()
    end

  end
end
