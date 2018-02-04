defmodule Budget.Repo.Migrations.CreateAdjustments do
  use Ecto.Migration

  def change do
    create table(:adjustments) do
      add(:title, :string)
      add(:account_id, :integer)
      add(:total, :float)

      timestamps()
    end
  end
end
