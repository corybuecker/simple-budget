defmodule SimpleBudget.TokenAuth.Dummy do
  @behaviour SimpleBudget.TokenAuth

  @impl SimpleBudget.TokenAuth
  def verify_and_validate_token(token) do
    signer = Joken.Signer.create("HS256", "development-use-only")

    config = Joken.Config.default_claims()

    case Joken.verify_and_validate(config, token, signer) do
      {:ok, %{"email" => email}} ->
        user_valid?(email)

      {:error, error} ->
        {:error, error}

      _ ->
        {:error, "unknown error"}
    end
  end

  @impl SimpleBudget.TokenAuth
  def user_valid?(email) do
    case SimpleBudget.Users.get_user!(email) do
      %{email: ^email} -> {:ok, email}
      _ -> {:error, "could not validate user"}
    end
  end
end
