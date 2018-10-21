defmodule SimpleBudget.UsersTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Users
  alias SimpleBudget.Users.User

  describe "users" do
    @valid_attrs %{email: "test@user.com"}

    def user_fixture() do
      {:ok, user} =
        %User{}
        |> User.changeset(@valid_attrs)
        |> Repo.insert()

      user
    end

    test "get_user!/1 returns matching user" do
      user = user_fixture()
      assert Users.get_user!("test@user.com") == user
    end
  end
end
