defmodule Budget.Repo.Migrations.CreateSavings do
  use Ecto.Migration

  def change do
    create table(:savings) do
      add :title, :string
      add :amount, :float

      timestamps()
    end

  end
end
