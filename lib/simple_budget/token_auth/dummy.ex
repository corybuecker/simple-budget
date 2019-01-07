defmodule SimpleBudget.TokenAuth.Dummy do
  @behaviour SimpleBudget.TokenAuth

  @impl SimpleBudget.TokenAuth
  def user_valid?(email) do
    case Users.get_user!(email) do
      %{email: ^email} -> true
      _ -> false
    end
  end
end
