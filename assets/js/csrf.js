((xhr) => {
  const originalOpen = xhr.open
  const node = document.head.querySelector('meta[name="csrf"]')
  const token = node.content

  xhr.open = function (method, url) {
    const originalResult = originalOpen.apply(this, arguments)
    this.setRequestHeader("X-CSRF-Token", token)

    return originalResult
  }
})(XMLHttpRequest.prototype)
