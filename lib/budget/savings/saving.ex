defmodule Budget.Savings.Saving do
  use Ecto.Schema
  import Ecto.Changeset
  alias Budget.Savings.Saving

  schema "savings" do
    field(:amount, :float)
    field(:amount_cents, :integer)
    field(:title, :string)

    timestamps()
  end

  @doc false
  def changeset(%Saving{} = saving, attrs) do
    saving
    |> cast(attrs, [:title, :amount])
    |> validate_required([:title, :amount])
  end
end
