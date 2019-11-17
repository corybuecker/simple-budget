defmodule SimpleBudget.Savings.Saving do
  @moduledoc false
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Savings.Saving
  alias SimpleBudget.Users.User

  schema "savings" do
    field(:amount, :decimal, scale: 8, precision: 2)
    field(:title, :string)
    belongs_to :user, User
    timestamps()
  end

  @doc false
  def changeset(%Saving{} = saving, attrs) do
    saving
    |> cast(attrs, [:title, :amount, :user_id])
    |> validate_required([:title, :amount, :user_id])
    |> assoc_constraint(:user)
  end
end
