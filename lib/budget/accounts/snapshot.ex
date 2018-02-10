defmodule Budget.Accounts.Snapshot do
  use Ecto.Schema
  import Ecto.Changeset
  alias Budget.Accounts.Snapshot

  schema "snapshots" do
    field(:account_id, :integer)
    field(:after, :float)
    field(:before, :float)
    field(:after_cents, :integer)
    field(:before_cents, :integer)

    timestamps()
  end

  @doc false
  def changeset(%Snapshot{} = snapshot, attrs) do
    snapshot
    |> cast(attrs, [:account_id, :before, :after])
    |> validate_required([:account_id, :before, :after])
  end
end
