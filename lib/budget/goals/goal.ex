defmodule Budget.Goals.Goal do
  use Ecto.Schema
  import Ecto.Changeset
  alias Budget.Goals.Goal

  schema "goals" do
    field(:end_date, :date)
    field(:start_date, :date)
    field(:target, :decimal)
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
