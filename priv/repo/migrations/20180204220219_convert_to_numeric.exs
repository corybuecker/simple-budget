defmodule Budget.Repo.Migrations.ConvertAccountsToInteger do
  use Ecto.Migration

  def up do
    alter table(:accounts) do
      modify(:balance, :numeric, precision: 9, scale: 3)
    end

    alter table(:adjustments) do
      modify(:total, :numeric, precision: 9, scale: 3)
    end

    alter table(:snapshots) do
      modify(:before, :numeric, precision: 9, scale: 3)
      modify(:after, :numeric, precision: 9, scale: 3)
    end

    alter table(:goals) do
      modify(:target, :numeric, precision: 9, scale: 3)
    end

    alter table(:savings) do
      modify(:amount, :numeric, precision: 9, scale: 3)
    end
  end

  def down do
    alter table(:accounts) do
      modify(:balance, :float)
    end

    alter table(:adjustments) do
      modify(:total, :float)
    end

    alter table(:snapshots) do
      modify(:before, :float)
      modify(:after, :float)
    end

    alter table(:goals) do
      modify(:target, :float)
    end

    alter table(:savings) do
      modify(:amount, :float)
    end
  end
end
