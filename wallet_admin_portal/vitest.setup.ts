// jsdom does not implement the modal parts of HTMLDialogElement.
HTMLDialogElement.prototype.showModal = function showModal() {
  this.setAttribute('open', '')
}
HTMLDialogElement.prototype.close = function close() {
  this.removeAttribute('open')
  this.dispatchEvent(new Event('close'))
}

// jsdom does not implement reportError; tests that expect it spy on it.
globalThis.reportError ??= () => {}
