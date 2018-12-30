defmodule SimpleBudgetWeb.Router do
  use SimpleBudgetWeb, :router
  import SimpleBudgetWeb.Auth, only: [check_google_session: 2, check_google_session_api: 2]

  @csp "default-src 'self'; script-src 'self' 'nonce-7q1w9Jiyp2Kf0xrGOGtdQGaW3IljYiEQXzOe/ftW9Q0='; frame-src 'self' https://accounts.google.com; object-src 'none'; style-src 'self' https://stackpath.bootstrapcdn.com 'unsafe-inline'; connect-src 'self'"

  pipeline :browser do
    if Application.get_env(:simple_budget, :env) == :prod do
      plug Plug.SSL, rewrite_on: [:x_forwarded_proto], hsts: true
    end

    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery
    plug :put_secure_browser_headers, %{"content-security-policy" => @csp}
    plug :check_google_session
  end

  pipeline :api do
    if Application.get_env(:simple_budget, :env) == :prod do
      plug Plug.SSL, rewrite_on: [:x_forwarded_proto], hsts: true
    end

    plug :accepts, ["json"]
    plug :fetch_session
    plug :protect_from_forgery
    plug :put_secure_browser_headers, %{"content-security-policy" => @csp}
    plug :check_google_session_api
  end

  forward("/healthcheck", SimpleBudgetWeb.HealthcheckRouter)

  scope "/", SimpleBudgetWeb do
    pipe_through :browser

    get("/", PageController, :index)
    get("/accounts", PageController, :accounts)
    get("/goals", PageController, :goals)
    get("/savings", PageController, :savings)

    get("/login", LoginController, :index)
    post("/login", LoginController, :create)
  end

  scope "/api", SimpleBudgetWeb do
    pipe_through(:api)

    resources "/accounts", AccountController do
      resources("/adjustments", AdjustmentController)
      resources("/snapshots", SnapshotController, only: [:index, :show])
    end

    resources("/savings", SavingController)
    resources("/goals", GoalController)
    resources("/calculations", CalculationsController, only: [:index])
  end
end
