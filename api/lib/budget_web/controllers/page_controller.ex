defmodule BudgetWeb.PageController do
  use BudgetWeb, :controller

  def index(conn, _params) do
    conn
    |> render("index.html")
  end
end
