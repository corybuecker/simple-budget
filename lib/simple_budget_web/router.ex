defmodule SimpleBudgetWeb.Router do
  use SimpleBudgetWeb, :router

  import SimpleBudgetWeb.Auth,
    only: [check_authenticated_session: 2, check_authenticated_api_session: 2]

  forward("/healthcheck", SimpleBudgetWeb.HealthcheckRouter)

  pipeline :browser do
    plug Plug.SSL, rewrite_on: [:x_forwarded_proto]

    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery

    plug :put_secure_browser_headers, %{
      "content-security-policy" =>
        "default-src 'none'; script-src 'self' 'nonce-54a168223faae85076894ce271860de9'; style-src 'self'; img-src 'self'; frame-src 'self' https://accounts.google.com; connect-src 'self';"
    }
  end

  pipeline :authenticated_browser do
    plug :check_authenticated_session
  end

  pipeline :api do
    plug Plug.SSL, rewrite_on: [:x_forwarded_proto]

    plug :accepts, ["json"]
    plug :fetch_session
    plug :protect_from_forgery
    plug :check_authenticated_api_session
    plug :put_secure_browser_headers, %{"content-security-policy" => "default-src 'none';"}
  end

  scope "/auth", SimpleBudgetWeb do
    pipe_through :browser

    post("/token", TokenController, :create)

    get("/login", LoginController, :index)
    post("/login", LoginController, :create)
    get("/logout", LoginController, :logout)
  end

  scope "/api", SimpleBudgetWeb do
    pipe_through :api

    resources "/accounts", AccountController do
      resources("/adjustments", AdjustmentController)
      resources("/snapshots", SnapshotController, only: [:index, :show])
    end

    resources("/savings", SavingController)
    resources("/goals", GoalController)
    resources("/calculations", CalculationsController, only: [:index])
  end

  scope "/", SimpleBudgetWeb do
    pipe_through [:browser, :authenticated_browser]

    get("/", PageController, :index)
    get("/accounts", PageController, :accounts)
    get("/goals", PageController, :goals)
    get("/savings", PageController, :savings)
    get("/calculations", PageController, :calculations)
  end
end
