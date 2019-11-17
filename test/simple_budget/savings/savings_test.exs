defmodule SimpleBudget.SavingsTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Savings
  alias SimpleBudget.Users

  describe "savings" do
    alias SimpleBudget.Savings.Saving

    @valid_attrs %{amount: 120.55, title: "some title"}
    @invalid_attrs %{amount: nil}

    def user_fixture do
      {:ok, user} = Users.create_user(%{email: "test@test.com", password: "Test"})
      user.id
    end

    def saving_fixture(attrs \\ %{}) do
      attrs
      |> Enum.into(@valid_attrs)
      |> Savings.create_saving!()
    end

    test "list_savings/1 returns all savings" do
      saving = saving_fixture(%{user_id: user_fixture()})
      assert Savings.list_savings(saving.user_id) == [saving]
    end

    test "get_saving!/2 returns the saving with given id" do
      saving = saving_fixture()
      assert Savings.get_saving!(saving.user, saving.id) == saving
    end

    test "create_saving/1 with valid data creates a saving" do
      assert {:ok, %Saving{} = saving} = Savings.create_saving!(@valid_attrs, user_fixture())

      assert saving.amount == Decimal.from_float(120.55)
      assert saving.title == "some title"
    end

    test "create_saving/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Savings.create_saving(@invalid_attrs, user_fixture())
    end

    test "update_saving/2 with valid data updates the saving" do
      saving = saving_fixture()

      assert {:ok, saving} =
               Savings.update_saving(saving, %{
                 amount: 456.75,
                 title: "some updated title",
                 user: saving.user
               })

      assert %Saving{} = saving
      assert saving.amount == Decimal.from_float(456.75)
      assert saving.title == "some updated title"
    end

    test "update_saving/2 with invalid data returns error changeset" do
      saving = saving_fixture(%{user_id: user_fixture()})
      update_changeset = %{amount: nil}
      assert {:error, %Ecto.Changeset{}} = Savings.update_saving(saving, update_changeset)
      assert saving == Savings.get_saving!(saving.user, saving.id)
    end

    test "delete_saving/1 deletes the saving" do
      saving = saving_fixture()
      assert {:ok, %Saving{}} = Savings.delete_saving(saving)
      assert_raise Ecto.NoResultsError, fn -> Savings.get_saving!(saving.user, saving.id) end
    end

    test "change_saving/1 returns a saving changeset" do
      saving = saving_fixture()
      assert %Ecto.Changeset{} = Savings.change_saving(saving)
    end
  end
end
