import { Controller } from '@hotwired/stimulus'
import Decimal from 'decimal.js'

export default class CurrencyInputController extends Controller<HTMLInputElement> {
  declare readonly precisionValue: number
  declare readonly hasPrecisionValue: boolean
  declare readonly inputTarget: HTMLInputElement
  declare readonly outputTarget: HTMLElement

  static targets = ["input", "output"]
  static values = {
    precision: Number
  }

  connect() {
    this.render()
  }

  change() {
    this.render()
  }

  private render() {
    const value = this.inputTarget.value
    const decimalValue = value !== '' && !isNaN(Number(value)) ? new Decimal(value) : new Decimal(0)
    this.outputTarget.innerText = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: this.hasPrecisionValue ? this.precisionValue : 0 }).format(decimalValue.toNumber())
  }
}
