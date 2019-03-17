defmodule SimpleBudgetWeb.Router do
  use SimpleBudgetWeb, :router

  import SimpleBudgetWeb.Auth,
    only: [check_authenticated_session: 2, check_authenticated_api_session: 2]

  @csp "default-src 'self'; script-src 'self' 'unsafe-eval' 'nonce-7q1w9Jiyp2Kf0xrGOGtdQGaW3IljYiEQXzOe/ftW9Q0='; frame-src 'self' https://accounts.google.com; object-src 'none'; style-src 'self' 'unsafe-inline'; connect-src 'self'"

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery
    plug :put_secure_browser_headers, %{"content-security-policy" => @csp}
  end

  pipeline :authenticated_browser do
    plug :check_authenticated_session
  end

  pipeline :api do
    plug :accepts, ["json"]
    plug :fetch_session
    plug :protect_from_forgery
    plug :check_authenticated_api_session
  end

  forward("/healthcheck", SimpleBudgetWeb.HealthcheckRouter)

  scope "/auth", SimpleBudgetWeb do
    pipe_through :browser

    post("/token", TokenController, :create)
    get("/login", LoginController, :index)
    post("/login", LoginController, :create)
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
  end
end
