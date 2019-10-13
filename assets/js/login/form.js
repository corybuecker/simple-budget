import React, { useState } from "react";
import { csrfToken } from "../util";

export const Form = function () {
  const [username, setUsername] = useState();
  const [password, setPassword] = useState();

  const getToken = async function (username, password) {
    const response = await fetch("/auth/token", {
      method: "POST",
      body: JSON.stringify({ email: username, password: password }),
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-Token": csrfToken()
      }
    });
    const { token } = await response.json();
    fetch("/auth/login", {
      method: "POST",
      body: JSON.stringify({ token: token }),
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-Token": csrfToken()
      }
    });
  };

  return (
    <div>
      <form method="POST">
        <input
          type="email"
          name="email"
          onChange={e => setUsername(e.target.value)}
        />
        <input
          type="password"
          name="password"
          onChange={e => setPassword(e.target.value)}
        />
        <button
          type="submit"
          onClick={e => {
            e.preventDefault();
            e.stopPropagation();
            getToken(username, password);
          }}
        >
          Login
        </button>
      </form>
    </div>
  );
};
