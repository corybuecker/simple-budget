defmodule SimpleBudgetWeb.SnapshotController do
  use SimpleBudgetWeb, :controller

  alias SimpleBudget.Accounts

  action_fallback(SimpleBudgetWeb.FallbackController)

  def index(conn, _params) do
    snapshots = Accounts.list_snapshots()
    render(conn, "index.json", snapshots: snapshots)
  end
end
