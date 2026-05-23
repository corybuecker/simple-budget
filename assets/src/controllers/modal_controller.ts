import { Controller } from '@hotwired/stimulus'

export default class ModalController extends Controller {
  close() {
    this.element.removeAttribute("src")
    this.element.replaceChildren()
  }
}
