defmodule SimpleBudget.Users do
  @moduledoc false
  import Ecto.Query, warn: false
  alias SimpleBudget.Repo

  alias SimpleBudget.Users.User

  def get_user(id) do
    email_query = from u in User, where: u.id == ^id

    email_query
    |> Repo.one!()
  end

  def get_user!(email) do
    email_query =
      from(u in User,
        where: u.email == ^email
      )

    email_query
    |> first()
    |> Repo.one()
  end

  def get_password(email) do
    email_query =
      from(u in User,
        where: u.email == ^email
      )

    case email_query |> first() |> Repo.one() do
      %User{password: password} -> {:ok, password}
      _ -> {:error, "missing user"}
    end
  end

  def create_user(attrs \\ %{}) do
    %User{}
    |> User.changeset(attrs)
    |> Repo.insert()
  end
end
