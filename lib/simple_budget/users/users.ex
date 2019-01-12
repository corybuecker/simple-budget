defmodule SimpleBudget.Users do
  @moduledoc false
  import Ecto.Query, warn: false
  alias SimpleBudget.Repo

  alias SimpleBudget.Users.User

  def get_user!(email) do
    email_query =
      from(u in User,
        where: u.email == ^email
      )

    email_query
    |> first()
    |> Repo.one()
  end

  def create_user(attrs \\ %{}) do
    %User{}
    |> User.changeset(attrs)
    |> Repo.insert()
  end
end
