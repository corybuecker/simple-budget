defmodule Budget.Savings do
  @moduledoc """
  The Savings context.
  """

  import Ecto.Query, warn: false
  alias Budget.Repo

  alias Budget.Savings.Saving

  @doc """
  Returns the list of savings.

  ## Examples

      iex> list_savings()
      [%Saving{}, ...]

  """
  def list_savings do
    Repo.all(Saving)
  end

  @doc """
  Gets a single saving.

  Raises `Ecto.NoResultsError` if the Saving does not exist.

  ## Examples

      iex> get_saving!(123)
      %Saving{}

      iex> get_saving!(456)
      ** (Ecto.NoResultsError)

  """
  def get_saving!(id), do: Repo.get!(Saving, id)

  @doc """
  Creates a saving.

  ## Examples

      iex> create_saving(%{field: value})
      {:ok, %Saving{}}

      iex> create_saving(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create_saving(attrs \\ %{}) do
    %Saving{}
    |> Saving.changeset(attrs)
    |> Repo.insert()
  end

  @doc """
  Updates a saving.

  ## Examples

      iex> update_saving(saving, %{field: new_value})
      {:ok, %Saving{}}

      iex> update_saving(saving, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update_saving(%Saving{} = saving, attrs) do
    saving
    |> Saving.changeset(attrs)
    |> Repo.update()
  end

  @doc """
  Deletes a Saving.

  ## Examples

      iex> delete_saving(saving)
      {:ok, %Saving{}}

      iex> delete_saving(saving)
      {:error, %Ecto.Changeset{}}

  """
  def delete_saving(%Saving{} = saving) do
    Repo.delete(saving)
  end

  @doc """
  Returns an `%Ecto.Changeset{}` for tracking saving changes.

  ## Examples

      iex> change_saving(saving)
      %Ecto.Changeset{source: %Saving{}}

  """
  def change_saving(%Saving{} = saving) do
    Saving.changeset(saving, %{})
  end
end
