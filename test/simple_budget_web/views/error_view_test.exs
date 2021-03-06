defmodule SimpleBudgetWeb.ErrorViewTest do
  use SimpleBudgetWeb.ConnCase, async: true

  # Bring render/3 and render_to_string/3 for testing custom views
  import Phoenix.View

  test "renders 404.html" do
    assert render_to_string(SimpleBudgetWeb.ErrorView, "404.html", []) == "Not Found"
  end

  test "render 500.html" do
    assert render_to_string(SimpleBudgetWeb.ErrorView, "500.html", []) == "Internal Server Error"
  end

  test "render any other" do
    assert render_to_string(SimpleBudgetWeb.ErrorView, "505.html", []) ==
             "HTTP Version Not Supported"
  end
end
