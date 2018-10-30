defmodule SimpleBudgetWeb.SnapshotView do
  use SimpleBudgetWeb, :view
  alias SimpleBudgetWeb.SnapshotView

  def render("index.json", %{snapshots: snapshots}) do
    %{data: render_many(snapshots, SnapshotView, "snapshot.json")}
  end

  def render("snapshot.json", %{snapshot: snapshot}) do
    %{
      id: snapshot.id,
      account_id: snapshot.account_id,
      before: snapshot.before,
      after: snapshot.after
    }
  end
end
