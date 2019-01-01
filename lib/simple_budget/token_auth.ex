defmodule SimpleBudget.TokenAuth do
  @callback config :: any
  @callback valid_user(String.t()) :: boolean
end
