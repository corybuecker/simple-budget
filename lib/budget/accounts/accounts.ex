defmodule Budget.Accounts do
  @moduledoc """
  The Accounts context.
  """

  import Ecto.Query, warn: false
  alias Budget.Repo

  alias Budget.Accounts.Account
  alias Ecto.Multi

  @doc """
  Returns the list of accounts.

  ## Examples

      iex> list_accounts()
      [%Account{}, ...]

  """
  def list_accounts do
    Repo.all(Account)
    |> Repo.preload(:adjustments)
  end

  @doc """
  Gets a single account.

  Raises `Ecto.NoResultsError` if the Account does not exist.

  ## Examples

      iex> get_account!(123)
      %Account{}

      iex> get_account!(456)
      ** (Ecto.NoResultsError)

  """
  def get_account!(id), do: Repo.get!(Account, id)

  @doc """
  Creates a account.

  ## Examples

      iex> create_account(%{field: value})
      {:ok, %Account{}}

      iex> create_account(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create_account(attrs \\ %{}) do
    account_changeset =
      %Account{}
      |> Account.changeset(attrs)

    account_changeset =
      case account_changeset.changes do
        %{balance: balance} when is_number(balance) ->
          Ecto.Changeset.change(account_changeset, %{balance_cents: round(balance * 100)})

        _ ->
          account_changeset
      end

    account_changeset |> Repo.insert()
  end

  @doc """
  Updates a account.

  ## Examples

      iex> update_account(account, %{field: new_value})
      {:ok, %Account{}}

      iex> update_account(account, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update_account(%Account{} = account, attrs) do
    account_changeset =
      account
      |> Account.changeset(attrs)

    account_changeset =
      case account_changeset.changes do
        %{balance: balance} when is_number(balance) ->
          Ecto.Changeset.change(account_changeset, %{balance_cents: round(balance * 100)})

        _ ->
          account_changeset
      end

    multi =
      Multi.new()
      |> Multi.update(:account, account_changeset)
      |> Multi.run(:snapshot, fn %{account: updated_account} ->
        create_snapshot(%{
          account_id: account.id,
          before: account.balance,
          after: updated_account.balance
        })
      end)

    case Repo.transaction(multi) do
      {:ok, %{account: account, snapshot: _snapshot}} -> {:ok, account}
      {:error, :account, changeset, %{}} -> {:error, changeset}
      {:error, :snapshot, changeset, %{}} -> {:error, changeset}
    end
  end

  @doc """
  Deletes a Account.

  ## Examples

      iex> delete_account(account)
      {:ok, %Account{}}

      iex> delete_account(account)
      {:error, %Ecto.Changeset{}}

  """
  def delete_account(%Account{} = account) do
    Repo.delete(account)
  end

  @doc """
  Returns an `%Ecto.Changeset{}` for tracking account changes.

  ## Examples

      iex> change_account(account)
      %Ecto.Changeset{source: %Account{}}

  """
  def change_account(%Account{} = account) do
    Account.changeset(account, %{})
  end

  alias Budget.Accounts.Adjustment

  @doc """
  Returns the list of adjustments.

  ## Examples

      iex> list_adjustments()
      [%Adjustment{}, ...]

  """
  def list_adjustments do
    Repo.all(Adjustment)
  end

  @doc """
  Gets a single adjustments.

  Raises `Ecto.NoResultsError` if the Adjustment does not exist.

  ## Examples

      iex> get_adjustments!(123)
      %Adjustment{}

      iex> get_adjustments!(456)
      ** (Ecto.NoResultsError)

  """
  def get_adjustment!(id), do: Repo.get!(Adjustment, id)

  @doc """
  Creates a adjustments.

  ## Examples

      iex> create_adjustment(%{field: value})
      {:ok, %Adjustment{}}

      iex> create_adjustment(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create_adjustment(attrs \\ %{}) do
    %Adjustment{}
    |> Adjustment.changeset(attrs)
    |> Repo.insert()
  end

  @doc """
  Updates a adjustments.

  ## Examples

      iex> update_adjustments(adjustment, %{field: new_value})
      {:ok, %Adjustment{}}

      iex> update_adjustments(adjustment, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update_adjustment(%Adjustment{} = adjustment, attrs) do
    adjustment
    |> Adjustment.changeset(attrs)
    |> Repo.update()
  end

  @doc """
  Deletes a Adjustment.

  ## Examples

      iex> delete_adjustments(adjustment)
      {:ok, %Adjustment{}}

      iex> delete_adjustments(adjustment)
      {:error, %Ecto.Changeset{}}

  """
  def delete_adjustment(%Adjustment{} = adjustment) do
    Repo.delete(adjustment)
  end

  @doc """
  Returns an `%Ecto.Changeset{}` for tracking adjustments changes.

  ## Examples

      iex> change_adjustments(adjustment)
      %Ecto.Changeset{source: %Adjustment{}}

  """
  def change_adjustment(%Adjustment{} = adjustment) do
    Adjustment.changeset(adjustment, %{})
  end

  alias Budget.Accounts.Snapshot

  @doc """
  Returns the list of snapshots.

  ## Examples

      iex> list_snapshots()
      [%Snapshot{}, ...]

  """
  def list_snapshots do
    Repo.all(Snapshot)
  end

  @doc """
  Gets a single snapshot.

  Raises `Ecto.NoResultsError` if the Snapshot does not exist.

  ## Examples

      iex> get_snapshot!(123)
      %Snapshot{}

      iex> get_snapshot!(456)
      ** (Ecto.NoResultsError)

  """
  def get_snapshot!(id), do: Repo.get!(Snapshot, id)

  @doc """
  Creates a snapshot.

  ## Examples

      iex> create_snapshot(%{field: value})
      {:ok, %Snapshot{}}

      iex> create_snapshot(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create_snapshot(attrs \\ %{}) do
    %Snapshot{}
    |> Snapshot.changeset(attrs)
    |> Repo.insert()
  end

  @doc """
  Updates a snapshot.

  ## Examples

      iex> update_snapshot(snapshot, %{field: new_value})
      {:ok, %Snapshot{}}

      iex> update_snapshot(snapshot, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update_snapshot(%Snapshot{} = snapshot, attrs) do
    snapshot
    |> Snapshot.changeset(attrs)
    |> Repo.update()
  end

  @doc """
  Deletes a Snapshot.

  ## Examples

      iex> delete_snapshot(snapshot)
      {:ok, %Snapshot{}}

      iex> delete_snapshot(snapshot)
      {:error, %Ecto.Changeset{}}

  """
  def delete_snapshot(%Snapshot{} = snapshot) do
    Repo.delete(snapshot)
  end

  @doc """
  Returns an `%Ecto.Changeset{}` for tracking snapshot changes.

  ## Examples

      iex> change_snapshot(snapshot)
      %Ecto.Changeset{source: %Snapshot{}}

  """
  def change_snapshot(%Snapshot{} = snapshot) do
    Snapshot.changeset(snapshot, %{})
  end
end
