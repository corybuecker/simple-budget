import '../csrf'
import '../normalize.css'
import './index.scss'
import { Elm } from './Main.elm'

class Login {
  constructor({ type }) {
    this.type = type
    console.log(type)
  }

  render() {
    if (this.type == 'email') {
      Elm.Main.init({
        node: document.getElementById('login'),
        flags: this.useEmail
      })
    }
  }
}

window.Login = Login
