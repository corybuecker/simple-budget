defmodule SimpleBudget.TokenAuth.Dummy do
  @behaviour SimpleBudget.TokenAuth

  use Joken.Config, default_signer: :pem_rs256

  @impl Joken.Config
  def token_config do
    default_claims()
  end

  @impl SimpleBudget.TokenAuth
  def verify_and_validate_token(token) do
  end

  @impl SimpleBudget.TokenAuth
  def user_valid?(email) do
    case Users.get_user!(email) do
      %{email: ^email} -> true
      _ -> false
    end
  end
end
