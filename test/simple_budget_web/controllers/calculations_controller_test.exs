defmodule SimpleBudgetWeb.CalculationsControllerTest do
  use SimpleBudgetWeb.ConnCase
  use Plug.Test

  setup %{conn: conn} do
    {:ok,
     conn:
       conn
       |> init_test_session(%{token: "validid"})
       |> put_req_header("accept", "application/json")}
  end

  describe "index" do
    test "lists calculations", %{conn: conn} do
      conn = get(conn, Routes.calculations_path(conn, :index))

      assert json_response(conn, 200)["data"] == %{"remaining" => 0.0, "remaining_per_day" => 0.0}
    end
  end
end
