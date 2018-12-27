import './app.scss'

// webpack automatically bundles all modules in your
// entry points. Those entry points can be configured
// in "webpack.config.js".
//
// Import dependencies
//
import "phoenix_html"

// Import local files
//
// Local files can be imported directly using relative paths, for example:
// import socket from "./socket"
import "./csrf"
import Elm from './Main.elm'

Elm.Elm.Main.init({
    node:
        document.getElementById('main')
});