import "@hotwired/turbo";
import { Application } from "@hotwired/stimulus";

import FormatterController from "./controllers/formatter_controller.js";
import ModalController from "./controllers/modal_controller.js";
import CurrencyInputController from "./controllers/currency_input_controller.js";

window.Stimulus = Application.start();

Stimulus.register("modal", ModalController);
Stimulus.register("formatter", FormatterController);
Stimulus.register("currency-input", CurrencyInputController);
