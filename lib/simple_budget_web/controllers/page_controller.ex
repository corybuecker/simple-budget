defmodule SimpleBudgetWeb.PageController do
  use SimpleBudgetWeb, :controller

  def index(conn, _params) do
    render(conn, "index.html")
  end
end
