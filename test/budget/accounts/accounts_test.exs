defmodule SimpleBudget.AccountsTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Accounts

  describe "accounts" do
    alias SimpleBudget.Accounts.Account
    alias SimpleBudget.Accounts.Snapshot

    @valid_attrs %{name: "some name", balance: 123.13, debt: false}
    @update_attrs %{name: "some updated name", balance: 456.42, debt: true}
    @invalid_attrs %{name: nil}

    def account_fixture(attrs \\ %{}) do
      {:ok, account} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Accounts.create_account()

      account
    end

    test "list_accounts/0 returns all accounts" do
      account = account_fixture()
      assert Accounts.list_accounts() == [account |> Repo.preload(:adjustments)]
    end

    test "get_account!/1 returns the account with given id" do
      account = account_fixture()
      account_query = Accounts.get_account!(account.id)
      assert account_query |> Repo.preload(:adjustments) == account
    end

    test "create_account/1 with valid data creates a account" do
      assert {:ok, %Account{} = account} = Accounts.create_account(@valid_attrs)
      assert account.name == "some name"
    end

    test "create_account/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Accounts.create_account(@invalid_attrs)
    end

    test "update_account/2 with valid data updates the account" do
      account = account_fixture()
      assert {:ok, account} = Accounts.update_account(account, @update_attrs)
      assert %Account{} = account
      assert account.name == "some updated name"
    end

    test "update_account/2 with valid data creates a snapshot" do
      account = account_fixture()
      assert {:ok, account} = Accounts.update_account(account, @update_attrs)
      assert %Account{} = account
      assert account.name == "some updated name"
      account_id = account.id
      expected_before = Decimal.from_float(123.13)
      expected_after = Decimal.from_float(456.42)

      assert [
               %Snapshot{
                 account_id: ^account_id,
                 before: ^expected_before,
                 after: ^expected_after
               }
             ] = Accounts.list_snapshots()
    end

    test "update_account/2 with invalid data returns error changeset" do
      account = account_fixture()
      assert {:error, %Ecto.Changeset{}} = Accounts.update_account(account, @invalid_attrs)
      account_query = Accounts.get_account!(account.id)
      assert account == account_query |> Repo.preload(:adjustments)
    end

    test "update_account/2 with invalid data does not create a snapshot" do
      account = account_fixture()
      assert {:error, %Ecto.Changeset{}} = Accounts.update_account(account, @invalid_attrs)
      account_query = Accounts.get_account!(account.id)

      assert account == account_query |> Repo.preload(:adjustments)
      assert [] = Accounts.list_snapshots()
    end

    test "delete_account/1 deletes the account" do
      account = account_fixture()
      assert {:ok, %Account{}} = Accounts.delete_account(account)
      assert_raise Ecto.NoResultsError, fn -> Accounts.get_account!(account.id) end
    end

    test "change_account/1 returns a account changeset" do
      account = account_fixture()
      assert %Ecto.Changeset{} = Accounts.change_account(account)
    end
  end

  describe "adjustments" do
    alias SimpleBudget.Accounts.Adjustment

    @valid_attrs %{account_id: 42, total: 120.13, title: "test"}
    @update_attrs %{account_id: 43, total: 456.42}
    @invalid_attrs %{account_id: nil, total: nil}

    def adjustment_fixture(attrs \\ %{}) do
      {:ok, adjustment} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Accounts.create_adjustment()

      adjustment
    end

    test "list_adjustments/0 returns all adjustments" do
      adjustment = adjustment_fixture()
      assert Accounts.list_adjustments() == [adjustment]
    end

    test "get_adjustment!/1 returns the adjustment with given id" do
      adjustment = adjustment_fixture()
      assert Accounts.get_adjustment!(adjustment.id) == adjustment
    end

    test "create_adjustment/1 with valid data creates a adjustment" do
      assert {:ok, %Adjustment{} = adjustment} = Accounts.create_adjustment(@valid_attrs)
      assert adjustment.account_id == 42
      assert adjustment.total == Decimal.from_float(120.13)
    end

    test "create_adjustment/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Accounts.create_adjustment(@invalid_attrs)
    end

    test "update_adjustment/2 with valid data updates the adjustment" do
      adjustment = adjustment_fixture()
      assert {:ok, adjustment} = Accounts.update_adjustment(adjustment, @update_attrs)
      assert %Adjustment{} = adjustment
      assert adjustment.account_id == 43
      assert adjustment.total == Decimal.from_float(456.42)
    end

    test "update_adjustment/2 with invalid data returns error changeset" do
      adjustment = adjustment_fixture()

      assert {:error, %Ecto.Changeset{}} = Accounts.update_adjustment(adjustment, @invalid_attrs)

      assert adjustment == Accounts.get_adjustment!(adjustment.id)
    end

    test "delete_adjustment/1 deletes the adjustment" do
      adjustment = adjustment_fixture()
      assert {:ok, %Adjustment{}} = Accounts.delete_adjustment(adjustment)
      assert_raise Ecto.NoResultsError, fn -> Accounts.get_adjustment!(adjustment.id) end
    end

    test "change_adjustment/1 returns a adjustment changeset" do
      adjustment = adjustment_fixture()
      assert %Ecto.Changeset{} = Accounts.change_adjustment(adjustment)
    end
  end

  describe "snapshots" do
    alias SimpleBudget.Accounts.Snapshot

    @valid_attrs %{account_id: 42, after: 120.51, before: 120.52}
    @update_attrs %{account_id: 43, after: 456.71, before: 456.72}
    @invalid_attrs %{account_id: nil, after: nil, before: nil}

    def snapshot_fixture(attrs \\ %{}) do
      {:ok, snapshot} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Accounts.create_snapshot()

      snapshot
    end

    test "list_snapshots/0 returns all snapshots" do
      snapshot = snapshot_fixture()
      assert Accounts.list_snapshots() == [snapshot]
    end

    test "get_snapshot!/1 returns the snapshot with given id" do
      snapshot = snapshot_fixture()
      assert Accounts.get_snapshot!(snapshot.id) == snapshot
    end

    test "create_snapshot/1 with valid data creates a snapshot" do
      assert {:ok, %Snapshot{} = snapshot} = Accounts.create_snapshot(@valid_attrs)
      assert snapshot.account_id == 42
      assert snapshot.after == Decimal.from_float(120.51)
      assert snapshot.before == Decimal.from_float(120.52)
    end

    test "create_snapshot/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Accounts.create_snapshot(@invalid_attrs)
    end

    test "update_snapshot/2 with valid data updates the snapshot" do
      snapshot = snapshot_fixture()
      assert {:ok, snapshot} = Accounts.update_snapshot(snapshot, @update_attrs)
      assert %Snapshot{} = snapshot
      assert snapshot.account_id == 43
      assert snapshot.after == Decimal.from_float(456.71)
      assert snapshot.before == Decimal.from_float(456.72)
    end

    test "update_snapshot/2 with invalid data returns error changeset" do
      snapshot = snapshot_fixture()
      assert {:error, %Ecto.Changeset{}} = Accounts.update_snapshot(snapshot, @invalid_attrs)
      assert snapshot == Accounts.get_snapshot!(snapshot.id)
    end

    test "delete_snapshot/1 deletes the snapshot" do
      snapshot = snapshot_fixture()
      assert {:ok, %Snapshot{}} = Accounts.delete_snapshot(snapshot)
      assert_raise Ecto.NoResultsError, fn -> Accounts.get_snapshot!(snapshot.id) end
    end

    test "change_snapshot/1 returns a snapshot changeset" do
      snapshot = snapshot_fixture()
      assert %Ecto.Changeset{} = Accounts.change_snapshot(snapshot)
    end
  end
end
