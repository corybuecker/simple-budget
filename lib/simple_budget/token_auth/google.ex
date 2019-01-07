defmodule SimpleBudget.TokenAuth.Google do
  @behaviour SimpleBudget.TokenAuth

  use Joken.Config

  alias SimpleBudget.Users

  add_hook(JokenJwks, jwks_url: "https://www.googleapis.com/oauth2/v3/certs")

  def token_config do
    default_claims(
      aud: "77675101516-vhivh2hl3b52h8906hmuvs47fd1vbhup.apps.googleusercontent.com",
      iss: "accounts.google.com"
    )
    |> add_claim("email", nil, &user_valid?/1)
  end

  @impl SimpleBudget.TokenAuth
  def verify_and_validate_token(token) do
    verify_and_validate(token)
  end

  @impl SimpleBudget.TokenAuth
  def user_valid?(email) do
    case Users.get_user!(email) do
      %{email: ^email} -> true
      _ -> false
    end
  end
end
