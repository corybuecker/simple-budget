defmodule SimpleBudget.Repo.Migrations.AddSessionsTable do
  use Ecto.Migration

  def change do
    create table(:sessions) do
      add :token, :string, null: false
      add :user_id, references(:users, on_delete: :delete_all), null: false

      timestamps()
    end
  end
end
