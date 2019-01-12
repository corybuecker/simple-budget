defmodule SimpleBudget.TokenAuth do
  @moduledoc false

  @callback user_valid?(String.t()) :: boolean
  @callback verify_and_validate_token(String.t()) :: {:ok, any} | {:error, any}
end
