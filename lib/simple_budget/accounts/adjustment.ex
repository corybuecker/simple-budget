defmodule SimpleBudget.Accounts.Adjustment do
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Accounts.Adjustment
  alias SimpleBudget.Accounts.Account

  schema "adjustments" do
    field(:total, :decimal, scale: 8, precision: 2)
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
