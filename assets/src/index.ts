import "../input.css";
import "@hotwired/turbo";
import { Application } from "@hotwired/stimulus";

import FormatterController from "./controllers/formatter_controller.js";
import ModalController from "./controllers/modal_controller.js";
import CurrencyInputController from "./controllers/currency_input_controller.js";

declare global {
  interface Window {
    Stimulus: Application;
  }
}

window.Stimulus = Application.start();

window.Stimulus.register("modal", ModalController);
window.Stimulus.register("formatter", FormatterController);
window.Stimulus.register("currency-input", CurrencyInputController);
