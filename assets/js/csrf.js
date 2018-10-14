(function (xhr) {
  let node, token;
  let originalOpen = xhr.open;

  node = document.head.querySelector('meta[name="csrf"]')
  token = node.content;

  xhr.open = function (method, url) {
    let originalResult = originalOpen.apply(this, arguments);
    this.setRequestHeader("X-CSRF-Token", token);
    return originalResult;
  }

})(XMLHttpRequest.prototype);