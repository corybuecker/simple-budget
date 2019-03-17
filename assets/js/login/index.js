import '../csrf'
import '../normalize.css'
import './index.scss'
import { Elm } from './Main.elm'

class Login {
  constructor({ ssoEnabled }) {
    this.ssoEnabled = ssoEnabled == true
  }

  render() {
    this.app = Elm.Main.init({
      node: document.getElementById('login'),
      flags: this.ssoEnabled
    })
  }

  renderGoogleAuth() {
    const googleAuth = gapi.auth2.getAuthInstance()

    this.app.ports.login.subscribe(() => {
      googleAuth.signIn()
    })

    if (googleAuth.isSignedIn.get()) {
      this.app.ports.useIdToken.send(googleAuth.currentUser.get().getAuthResponse().id_token)
    }
  }
}

window.Login = Login
