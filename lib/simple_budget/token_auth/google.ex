defmodule SimpleBudget.TokenAuth.Google do
  defmodule Strategy do
    use JokenJwks.DefaultStrategyTemplate

    def init_opts(opts) do
      Keyword.merge(opts, jwks_url: "https://www.googleapis.com/oauth2/v3/certs")
    end
  end

  @moduledoc false

  @callback user_valid?(String.t()) :: boolean
  @callback verify_and_validate_token(String.t()) :: {:ok, any} | {:error, any}

  use Joken.Config

  alias SimpleBudget.Users

  add_hook(JokenJwks, strategy: Strategy)

  def token_config do
    default_claims(
      aud: Application.fetch_env!(:simple_budget, :google_client_id),
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
