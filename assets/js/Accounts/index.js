import "phoenix_html"
import "../csrf"

import { Elm } from './Main.elm'

Elm.Main.init({
  node: document.getElementById('main')
});