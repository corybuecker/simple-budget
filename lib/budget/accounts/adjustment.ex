defmodule Budget.Accounts.Adjustment do
  use Ecto.Schema
  import Ecto.Changeset
  alias Budget.Accounts.Adjustment
  alias Budget.Accounts.Account

  schema "adjustments" do
    field(:total, :decimal)
    field(:title, :string)
    timestamps()

    belongs_to(:account, Account)
  end

  @doc false
  def changeset(%Adjustment{} = adjustments, attrs) do
    adjustments
    |> cast(attrs, [:account_id, :total, :title])
    |> validate_required([:account_id, :total, :title])
  end
end
