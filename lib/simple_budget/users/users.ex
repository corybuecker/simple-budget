defmodule SimpleBudget.Users do
  import Ecto.Query, warn: false
  alias SimpleBudget.Repo

  alias SimpleBudget.Users.User

  def get_user!(email) do
    from(u in User,
      where: u.email == ^email
    )
    |> first()
    |> Repo.one()
  end
end
