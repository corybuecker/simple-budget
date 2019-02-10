defmodule SimpleBudget.Users.User do
  @moduledoc false
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Users.User

  schema "users" do
    field(:email, :string)
    field(:password, :string)

    timestamps()
  end

  @doc false
  def changeset(%User{} = user, attrs) do
    user
    |> cast(attrs, [:email, :password])
    |> validate_required([:email, :password])
  end
end
