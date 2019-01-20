import { View } from 'backbone'
import $ from 'jquery'
import { template } from 'underscore'

import '../csrf'
import './index.css'

const form = () => {
  return template(`<form novalidate class='login-form'></form>`)
}

const usernameInput = () => {
  return template(`
    <label for='username'>
      Username:
      <input required autocapitalize='off' autocomplete='off' minlength='3' aria-invalid='false' type='text' name='username'></input>
    </label>
  `)
}

const usernameReadonly = () => {
  return template(`
    <label for='username'>
      Username:
      <input required readonly='true' autocapitalize='off' autocomplete='off' aria-invalid='false' minlength='3' type='text' name='username'></input>
    </label>
  `)
}

const password = () => {
  return template(`
    <label for='password'>
      Password:
      <input name='password' type='password' minlength='4' required='true' aria-invalid='false' autofocus></input>
    </label>
  `)
}

const fakePassword = () => {
  return template(`
    <input name='password' tabindex='-1' type='password' aria-hidden='true' hidden='true'></input>
  `)
}

const usernameSubmit = () => {
  return template(`
    <button class='username-submit' type='submit'>Next</button>
  `)
}

const passwordSubmit = () => {
  return template(`
    <button class='password-submit' type='submit'>Log in</button>
  `)
}

class GoogleLogin extends View {
  initialize(options) {
    this.options = options
  }

  render() {
    return this.$el
  }
}

class DummyLogin extends View {
  get events() {
    return {
      'submit form': (e) => e.preventDefault(),
      'click button.username-submit': (e) => this.nextStep(e),
      'click button.password-submit': (e) => this.commit(e)
    }
  }

  initialize(options) {
    this.options = options

    this.usernameInput = usernameInput()
    this.usernameSubmit = usernameSubmit()
    this.usernameReadonly = usernameReadonly()
    this.password = password()
    this.fakePassword = fakePassword()
    this.passwordSubmit = passwordSubmit()
    this.form = form()
  }

  nextStep(e) {
    e.preventDefault()

    const username = this.$('input').val()
    this.$el.empty()

    this.$el.append(this.form())
    this.renderedForm = this.$('form')

    this.renderedForm.append(this.usernameReadonly())
    this.renderedForm.append(this.password())
    this.renderedForm.append(this.passwordSubmit())

    this.$('input[name="username"]').val(username)
  }

  commit(e) {
    e.preventDefault()

    const username = this.$('input[name="username"]').val()
    const password = this.$('input[type="password"]').val()

    fetch("/login", {
      method: "POST",
      mode: "same-origin",
      cache: "no-cache",
      credentials: "same-origin",
      headers: {
        "Content-Type": "application/json"
      },
      redirect: "error",
      referrer: "no-referrer",
      body: JSON.stringify({ 'username': username, 'password': password })
    }).then((response) => {
      window.location = '/accounts'
    })
  }

  render() {
    this.$el.append(this.form())
    this.renderedForm = this.$('form')

    this.renderedForm.append(this.usernameInput())
    this.renderedForm.append(this.fakePassword())
    this.renderedForm.append(this.usernameSubmit())

    return this.$el
  }
}

class Login extends View {
  initialize(options) {
    this.options = options
    this.viewClass = {
      'dummy': DummyLogin,
      'google': GoogleLogin
    }[options.type]
  }

  render() {
    const view = new this.viewClass(this.options)

    $('.login').append(view.render())

    return this.$el
  }
}

window.Login = Login