defmodule SimpleBudget.TokenAuth.Google do
  @moduledoc false

  @callback user_valid?(String.t()) :: boolean
  @callback verify_and_validate_token(String.t()) :: {:ok, any} | {:error, any}

  use Joken.Config

  alias SimpleBudget.Users

  add_hook(JokenJwks, jwks_url: "https://www.googleapis.com/oauth2/v3/certs")

  def token_config do
    default_claims(
      aud: "77675101516-vhivh2hl3b52h8906hmuvs47fd1vbhup.apps.googleusercontent.com",
      iss: "accounts.google.com"
    )
  end

  def verify_and_validate_token(token) do
    case verify_and_validate(token) do
      {:ok, %{"email" => email}} ->
        user_valid?(email)

      {:error, error} ->
        {:error, error}

      _ ->
        {:error, "unknown error"}
    end
  end

  def user_valid?(email) do
    case Users.get_user!(email) do
      %{email: ^email} -> {:ok, email}
      _ -> {:error, "could not validate user"}
    end
  end
end
