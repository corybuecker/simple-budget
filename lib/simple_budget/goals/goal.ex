defmodule SimpleBudget.Goals.Goal do
  @moduledoc false
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Goals.Goal
  alias SimpleBudget.Users.User

  schema "goals" do
    field(:end_date, :date)
    field(:start_date, :date)
    field(:target, :decimal, scale: 8, precision: 2)
    field(:title, :string)
    belongs_to :user, User

    timestamps()
  end

  @doc false
  def changeset(%Goal{} = goal, attrs) do
    goal
    |> cast(attrs, [:title, :start_date, :end_date, :target])
    |> put_assoc(:user, attrs[:user])
    |> validate_required([:title, :start_date, :end_date, :target, :user])
  end
end
