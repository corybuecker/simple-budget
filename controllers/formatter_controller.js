import { Controller } from '@hotwired/stimulus'

export default class FormatterController extends Controller {
  static values = {
    currency: Number,
    currencyPrecision: Number
  }

  connect() {
    if(this.hasCurrencyValue) {
      this.element.innerText = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: this.hasCurrencyPrecisionValue ? this.currencyPrecisionValue : 0 }).format(this.currencyValue)
    }
  }
}
