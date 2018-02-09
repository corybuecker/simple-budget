defmodule Budget.Repo.Migrations.AddCents do
  use Ecto.Migration

  def change do
    alter table("accounts") do
      add(:balance_cents, :integer)
    end

    alter table("adjustments") do
      add(:total_cents, :integer)
    end

    alter table("snapshots") do
      add(:before_cents, :integer)
      add(:after_cents, :integer)
    end

    alter table("goals") do
      add(:target_cents, :integer)
    end

    alter table("savings") do
      add(:amount_cents, :integer)
    end
  end
end
