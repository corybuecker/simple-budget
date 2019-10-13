defmodule SimpleBudget.Accounts.Account do
  @moduledoc false
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Accounts.Account
  alias SimpleBudget.Users.User
  alias SimpleBudget.Accounts.Adjustment

  schema "accounts" do
    field(:name, :string)
    field(:balance, :decimal, scale: 8, precision: 2)
    field(:debt, :boolean)
    belongs_to :user, User
    timestamps()

    has_many(:adjustments, Adjustment)
  end

  @doc false
  def changeset(%Account{} = account, attrs) do
    account
    |> cast(attrs, [:name, :balance, :debt])
    |> put_assoc(:user, attrs[:user])
    |> validate_required([:name, :balance, :debt])
  end
end
