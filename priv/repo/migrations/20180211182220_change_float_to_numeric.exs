defmodule Budget.Repo.Migrations.ChangeFloatToNumeric do
  use Ecto.Migration

  def up do
    alter table("accounts") do
      modify(:balance, :decimal, precision: 8, scale: 2)
    end

    alter table("adjustments") do
      modify(:total, :decimal, precision: 8, scale: 2)
    end

    alter table("snapshots") do
      modify(:before, :decimal, precision: 8, scale: 2)
      modify(:after, :decimal, precision: 8, scale: 2)
    end

    alter table("goals") do
      modify(:target, :decimal, precision: 8, scale: 2)
    end

    alter table("savings") do
      modify(:amount, :decimal, precision: 8, scale: 2)
    end
  end

  def down do
    alter table("accounts") do
      modify(:balance, :float)
    end

    alter table("adjustments") do
      modify(:total, :float)
    end

    alter table("snapshots") do
      modify(:before, :float)
      modify(:after, :float)
    end

    alter table("goals") do
      modify(:target, :float)
    end

    alter table("savings") do
      modify(:amount, :float)
    end
  end
end
