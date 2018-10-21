defmodule SimpleBudget.TokenTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.Token

  describe "invalid token" do
    test "returns error" do
      {:error, message} =
        Token.verify_and_validate(
          "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
        )

      assert message == :kid_does_not_match
    end
  end
end
