defmodule SimpleBudget.Accounts do
  import Ecto.Query, warn: false

  alias Ecto.Multi
  alias SimpleBudget.Accounts.Account
  alias SimpleBudget.Repo
  alias SimpleBudget.Accounts.Adjustment
  alias SimpleBudget.Accounts.Snapshot

  def list_accounts(user_id) when is_integer(user_id) do
    query = from q in Account, where: q.user_id == ^user_id

    query
    |> Repo.all()
    |> Repo.preload(:adjustments)
  end

  def get_account!(user_id, id) when is_integer(user_id) and is_integer(id) do
    query = from q in Account, where: q.user_id == ^user_id, where: q.id == ^id

    query
    |> Repo.one!()
    |> Repo.preload(:adjustments)
  end

  def create_account(attrs \\ %{}) do
    case %Account{} |> Account.changeset(attrs) |> Repo.insert() do
      {:ok, account} ->
        {:ok, account |> Repo.preload(:adjustments)}

      {:error, account} ->
        {:error, account}
    end
  end

  def update_account(%Account{} = account, attrs) do
    account_changeset =
      account
      |> Account.changeset(attrs)

    multi =
      Multi.new()
      |> Multi.update(:account, account_changeset)
      |> Multi.run(:snapshot, fn _repo, %{account: updated_account} ->
        create_snapshot(%{
          account_id: account.id,
          before: account.balance,
          after: updated_account.balance
        })
      end)

    case Repo.transaction(multi) do
      {:ok, %{account: account, snapshot: _snapshot}} ->
        {:ok, account |> Repo.preload(:adjustments)}

      {:error, :account, changeset, %{}} ->
        {:error, changeset}

      {:error, :snapshot, changeset, %{}} ->
        {:error, changeset}
    end
  end

  def delete_account(%Account{} = account) do
    Repo.delete(account)
  end

  def change_account(%Account{} = account) do
    Account.changeset(account, %{})
  end

  def list_adjustments do
    Repo.all(Adjustment)
  end

  def get_adjustment!(id), do: Repo.get!(Adjustment, id)

  def create_adjustment(attrs \\ %{}) do
    %Adjustment{}
    |> Adjustment.changeset(attrs)
    |> Repo.insert()
  end

  def update_adjustment(%Adjustment{} = adjustment, attrs) do
    adjustment
    |> Adjustment.changeset(attrs)
    |> Repo.update()
  end

  def delete_adjustment(%Adjustment{} = adjustment) do
    Repo.delete(adjustment)
  end

  def change_adjustment(%Adjustment{} = adjustment) do
    Adjustment.changeset(adjustment, %{})
  end

  def list_snapshots do
    Repo.all(Snapshot)
  end

  def get_snapshot!(id), do: Repo.get!(Snapshot, id)

  def create_snapshot(attrs \\ %{}) do
    %Snapshot{}
    |> Snapshot.changeset(attrs)
    |> Repo.insert()
  end

  def update_snapshot(%Snapshot{} = snapshot, attrs) do
    snapshot
    |> Snapshot.changeset(attrs)
    |> Repo.update()
  end

  def delete_snapshot(%Snapshot{} = snapshot) do
    Repo.delete(snapshot)
  end

  def change_snapshot(%Snapshot{} = snapshot) do
    Snapshot.changeset(snapshot, %{})
  end
end
