defmodule SimpleBudget.TokenAuth.EmailTest do
  use SimpleBudget.DataCase

  alias SimpleBudget.TokenAuth.Email
  alias SimpleBudget.Users

  def user_fixture(attrs \\ %{}) do
    {:ok, user} = Users.create_user(attrs)

    user
  end

  describe "invalid token" do
    test "returns error" do
      {:error, message} =
        Email.verify_and_validate_token(
          "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
        )

      assert message == :signature_error
    end
  end

  describe "valid token without email claim" do
    test "returns error" do
      signer = Joken.Signer.create("HS256", "development-use-only")
      config = Joken.Config.default_claims()

      {:ok, token, _claims} = Joken.generate_and_sign(config, %{}, signer)

      {:error, message} = Email.verify_and_validate_token(token)

      assert message == "unknown error"
    end
  end

  describe "valid token with unknown user" do
    test "returns error" do
      signer = Joken.Signer.create("HS256", "development-use-only")
      config = Joken.Config.default_claims()

      {:ok, token, _claims} = Joken.generate_and_sign(config, %{email: "test@user.com"}, signer)

      {:error, message} = Email.verify_and_validate_token(token)

      assert message == "could not validate user"
    end
  end

  describe "valid token with matching user" do
    test "returns error" do
      user_fixture(%{email: "test@user.com", password: Argon2.hash_pwd_salt("password")})
      signer = Joken.Signer.create("HS256", "development-use-only")
      config = Joken.Config.default_claims()

      {:ok, token, _claims} = Joken.generate_and_sign(config, %{email: "test@user.com"}, signer)
      {:ok, email} = Email.verify_and_validate_token(token)
      assert email == "test@user.com"
    end
  end
end
