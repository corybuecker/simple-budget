defmodule SimpleBudget.Repo.Migrations.AddUserIdForeignKeys do
  use Ecto.Migration

  def change do
    alter table(:accounts) do
      add(:user_id, references(:users, on_delete: :delete_all), null: false)
    end

    alter table(:goals) do
      add(:user_id, references(:users, on_delete: :delete_all), null: false)
    end

    alter table(:savings) do
      add(:user_id, references(:users, on_delete: :delete_all), null: false)
    end
  end
end
