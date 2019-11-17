defmodule SimpleBudget.Accounts.Account do
  use Ecto.Schema
  import Ecto.Changeset

  alias SimpleBudget.Accounts.Account
  alias SimpleBudget.Users.User
  alias SimpleBudget.Accounts.Adjustment

  schema "accounts" do
    field(:name, :string)
    field(:balance, :decimal, scale: 8, precision: 2)
    field(:debt, :boolean)
    belongs_to(:user, User)
    timestamps()

    has_many(:adjustments, Adjustment)
  end

  def changeset(%Account{} = account, attrs) do
    account
    |> cast(attrs, [:name, :balance, :debt, :user_id])
    |> validate_required([:name, :balance, :debt, :user_id])
    |> assoc_constraint(:user)
  end
end
