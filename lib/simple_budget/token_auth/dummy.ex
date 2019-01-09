defmodule SimpleBudget.TokenAuth.Dummy do
  @behaviour SimpleBudget.TokenAuth

  use Joken.Config

  @impl SimpleBudget.TokenAuth
  def verify_and_validate_token(token) do
    signer = Joken.Signer.create("HS256", "development-use-only")

    config =
      Joken.Config.default_claims()
      |> Joken.Config.add_claim("email", nil, &user_valid?/1)

    Joken.verify_and_validate(config, token, signer)
  end

  @impl SimpleBudget.TokenAuth
  def user_valid?(email) do
    case SimpleBudget.Users.get_user!(email) do
      %{email: ^email} -> true
      _ -> false
    end
  end
end
