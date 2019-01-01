defmodule SimpleBudget.TokenAuth.Google do
  @behaviour SimpleBudget.TokenAuth
  # @moduledoc false
  use Joken.Config

  @impl SimpleBudget.TokenAuth
  def config do
    token_config()
  end

  alias SimpleBudget.Users

  add_hook(JokenJwks, jwks_url: "https://www.googleapis.com/oauth2/v3/certs")

  def token_config do
    default_claims(
      aud: "77675101516-vhivh2hl3b52h8906hmuvs47fd1vbhup.apps.googleusercontent.com",
      iss: "accounts.google.com"
    )
    |> add_claim("email", nil, &valid_user/1)
  end

  @impl SimpleBudget.TokenAuth
  def valid_user(email) do
    case Users.get_user!(email) do
      %{email: ^email} -> true
      _ -> false
    end
  end
end
