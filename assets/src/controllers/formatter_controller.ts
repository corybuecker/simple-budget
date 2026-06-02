import { Controller } from '@hotwired/stimulus'

export default class FormatterController extends Controller {
  declare readonly currencyValue: number
  declare readonly hasCurrencyValue: boolean
  declare readonly currencyPrecisionValue: number
  declare readonly hasCurrencyPrecisionValue: boolean

  static values = {
    currency: Number,
    currencyPrecision: Number
  }


  connect() {
    if(this.hasCurrencyValue) {
      this.element.textContent = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: this.hasCurrencyPrecisionValue ? this.currencyPrecisionValue : 0 }).format(this.currencyValue)
    }
  }
}
