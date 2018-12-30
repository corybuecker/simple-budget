defmodule SimpleBudgetWeb.PageController do
  use SimpleBudgetWeb, :controller

  def index(conn, _params) do
    render(conn, "index.html")
  end

  def accounts(conn, _params) do
    render(conn, "accounts.html")
  end

  def goals(conn, _params) do
    render(conn, "goals.html")
  end

  def savings(conn, _params) do
    render(conn, "savings.html")
  end
end
