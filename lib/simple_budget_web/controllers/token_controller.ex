defmodule SimpleBudgetWeb.TokenController do
  use SimpleBudgetWeb, :controller

  action_fallback SimpleBudgetWeb.FallbackController

  def create(conn, params) do
    case Application.get_env(:simple_budget, :authentication) do
      :dummy ->
        signer = Joken.Signer.create("HS256", "development-use-only")
        config = Joken.Config.default_claims()

        {:ok, token, _claims} = Joken.generate_and_sign(config, params, signer)

        render(conn, "create.json", token: token)

      _ ->
        conn
        |> send_resp(401, "{}")
        |> halt()
    end
  end
end
