defmodule BudgetWeb.Router do
  use BudgetWeb, :router

  pipeline :browser do
    plug(:accepts, ["html"])
    plug(:fetch_session)
    plug(:fetch_flash)
    # plug(:protect_from_forgery)
    plug(:put_secure_browser_headers)
  end

  pipeline :api do
    plug(:accepts, ["json"])
    plug(:fetch_session)
    # plug(:protect_from_forgery)
    plug(:put_secure_browser_headers)
  end

  scope "/", BudgetWeb do
    # Use the default browser stack
    pipe_through(:browser)

    get("/", PageController, :index)
    get("/accounts", PageController, :index)
    get("/goals", PageController, :index)
    get("/savings", PageController, :index)
    forward("/healthcheck", HealthcheckRouter)
  end

  # Other scopes may use custom stacks.
  scope "/api", BudgetWeb do
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
