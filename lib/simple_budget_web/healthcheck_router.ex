defmodule SimpleBudgetWeb.HealthcheckRouter do
  @moduledoc false
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  get "/" do
    send_resp(conn, 200, "")
  end
end
