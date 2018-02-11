defmodule Budget.Savings.Saving do
  use Ecto.Schema
  import Ecto.Changeset
  alias Budget.Savings.Saving

  schema "savings" do
    field(:amount, :decimal, scale: 8, precision: 2)
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
