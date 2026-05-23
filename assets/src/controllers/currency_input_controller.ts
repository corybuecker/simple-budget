import { Controller } from '@hotwired/stimulus'

export default class CurrencyInputController extends Controller {
  static targets = ["input", "output"]
  static values = {
    precision: Number
  }

  connect() {
    this.outputTarget.innerText = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: this.hasPrecisionValue ? this.precisionValue : 0 }).format(this.inputTarget.value)
  }

  change(event) {
    this.outputTarget.innerText = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: this.hasPrecisionValue ? this.precisionValue : 0 }).format(event.target.value)
  }
}
