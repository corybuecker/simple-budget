defmodule SimpleBudgetWeb.Layouts do
  use SimpleBudgetWeb, :html

  embed_templates "layouts/*"

  def navigation_class(expected_title, %{page_title: current_title}) do
    case expected_title == current_title do
      true ->
        "bg-slate-600 text-white lg:text-base text-sm rounded-md p-2"

      _ ->
        "text-white hover:bg-slate-600 lg:text-base text-sm rounded-md p-2"
    end
  end
end
