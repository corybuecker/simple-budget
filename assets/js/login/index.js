import '../csrf'
import '../normalize.css'
import './index.scss'
import { Elm } from './Main.elm'

class Login {
  constructor({ type }) {
    this.useDummy = type == 'dummy'
  }

  render() {
    Elm.Main.init({
      node: document.getElementById('login'),
      flags: this.useDummy
    })
  }
}

window.Login = Login
