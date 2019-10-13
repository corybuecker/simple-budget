defmodule SimpleBudgetWeb.Router do
  use SimpleBudgetWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
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
    pipe_through :browser

    get("/", PageController, :index)
    get("/accounts", PageController, :accounts)
    get("/goals", PageController, :goals)
    get("/savings", PageController, :savings)
    get("/calculations", PageController, :calculations)
  end
end
