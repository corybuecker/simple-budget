defmodule BudgetWeb.SnapshotView do
  use BudgetWeb, :view
  alias BudgetWeb.SnapshotView

  def render("index.json", %{snapshots: snapshots}) do
    %{data: render_many(snapshots, SnapshotView, "snapshot.json")}
  end

  def render("show.json", %{snapshot: snapshot}) do
    %{data: render_one(snapshot, SnapshotView, "snapshot.json")}
  end

  def render("snapshot.json", %{snapshot: snapshot}) do
    %{
      id: snapshot.id,
      account_id: snapshot.account_id,
      before: snapshot.before,
      before_cents: snapshot.before * 100.0,
      after_cents: snapshot.after * 100.0,
      after: snapshot.after
    }
  end
end
