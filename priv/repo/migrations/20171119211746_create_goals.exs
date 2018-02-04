defmodule Budget.Repo.Migrations.CreateGoals do
  use Ecto.Migration

  def change do
    create table(:goals) do
      add(:title, :string)
      add(:start_date, :date)
      add(:end_date, :date)
      add(:target, :float)

      timestamps()
    end
  end
end
