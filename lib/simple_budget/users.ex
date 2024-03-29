defmodule SimpleBudget.Users do
  import Ecto.Query
  alias SimpleBudget.{Repo, User}

  def existing_identity?(identity) do
    User |> Ecto.Query.where(identity: ^identity) |> Repo.exists?()
  end

  def get_by_email(email) do
    Repo.one(from u in User, where: u.email == ^email, select: u)
  end

  @spec get_by_identity(%{required(String.t()) => String.t()}) :: nil | SimpleBudget.User.t()
  def get_by_identity(%{"identity" => identity}) do
    Repo.one(from u in User, where: u.identity == ^identity, select: u)
  end

  def get_by_identity(identity) do
    Repo.one(from u in User, where: u.identity == ^identity, select: u)
  end

  def update(%User{} = user, params) do
    user |> User.changeset(params) |> Repo.update()
  end

  def reload(%User{id: id} = user) when not is_nil(id) do
    user |> Repo.reload()
  end
end
