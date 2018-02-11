defmodule Budget.Repo.Migrations.RemoveCents do
  use Ecto.Migration

  def change do
    alter table("accounts") do
      remove(:balance_cents)
    end

    alter table("adjustments") do
      remove(:total_cents)
    end

    alter table("snapshots") do
      remove(:before_cents)
      remove(:after_cents)
    end

    alter table("goals") do
      remove(:target_cents)
    end

    alter table("savings") do
      remove(:amount_cents)
    end
  end
end
