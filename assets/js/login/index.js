import '../csrf'
import '../normalize.css'
import './index.scss'
import { Elm } from './Main.elm'

class Login {
  constructor({ ssoEnabled }) {
    this.ssoEnabled = ssoEnabled == true
  }

  render() {
    Elm.Main.init({
      node: document.getElementById('login'),
      flags: this.ssoEnabled
    })
  }
}

window.Login = Login
