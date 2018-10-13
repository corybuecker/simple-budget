defmodule SimpleBudget.Goals.Goal do
  use Ecto.Schema
  import Ecto.Changeset
  alias SimpleBudget.Goals.Goal

  schema "goals" do
    field(:end_date, :date)
    field(:start_date, :date)
    field(:target, :decimal, scale: 8, precision: 2)
    field(:title, :string)

    timestamps()
  end

  @doc false
  def changeset(%Goal{} = goal, attrs) do
    goal
    |> cast(attrs, [:title, :start_date, :end_date, :target])
    |> validate_required([:title, :start_date, :end_date, :target])
  end
end
