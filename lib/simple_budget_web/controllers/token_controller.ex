defmodule SimpleBudgetWeb.TokenController do
  use SimpleBudgetWeb, :controller
  alias SimpleBudget.Users

  action_fallback SimpleBudgetWeb.FallbackController

  def create(conn, params) do
    with :email <- Application.get_env(:simple_budget, :authentication),
         {:ok, password} <- Users.get_password(params["email"]),
         true <- Argon2.verify_pass(params["password"], password) do
      signer = Joken.Signer.create("HS256", Application.get_env(:simple_budget, :token_key))
      config = Joken.Config.default_claims()

      {:ok, token, _claims} = Joken.generate_and_sign(config, params, signer)

      render(conn, "create.json", token: token)
    else
      _ ->
        conn
        |> put_resp_header("content-type", "application/json")
        |> send_resp(401, "{}")
        |> halt()
    end
  end
end
