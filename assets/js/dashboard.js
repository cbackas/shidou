/* global htmx */

function isValidUrl (input) {
  input = input.trim()
  input = input.includes('://') ? input : 'http://' + input
  const reWebURL = new RegExp(
    '^' +
    // protocol identifier (optional)
    // short syntax // still required
    '(?:(?:(?:https?|ftp):)?\\/\\/)' +
    // user:pass BasicAuth (optional)
    '(?:\\S+(?::\\S*)?@)?' +
    '(?:' +
    // IP address exclusion
    // private & local networks
    '(?!(?:10|127)(?:\\.\\d{1,3}){3})' +
    '(?!(?:169\\.254|192\\.168)(?:\\.\\d{1,3}){2})' +
    '(?!172\\.(?:1[6-9]|2\\d|3[0-1])(?:\\.\\d{1,3}){2})' +
    // IP address dotted notation octets
    // excludes loopback network 0.0.0.0
    // excludes reserved space >= 224.0.0.0
    // excludes network & broadcast addresses
    // (first & last IP address of each class)
    '(?:[1-9]\\d?|1\\d\\d|2[01]\\d|22[0-3])' +
    '(?:\\.(?:1?\\d{1,2}|2[0-4]\\d|25[0-5])){2}' +
    '(?:\\.(?:[1-9]\\d?|1\\d\\d|2[0-4]\\d|25[0-4]))' +
    '|' +
    // host & domain names, may end with dot
    // can be replaced by a shortest alternative
    // (?![-_])(?:[-\\w\\u00a1-\\uffff]{0,63}[^-_]\\.)+
    '(?:' +
    '(?:' +
    '[a-z0-9\\u00a1-\\uffff]' +
    '[a-z0-9\\u00a1-\\uffff_-]{0,62}' +
    ')?' +
    '[a-z0-9\\u00a1-\\uffff]\\.' +
    ')+' +
    // TLD identifier name, may end with dot
    '(?:[a-z\\u00a1-\\uffff]{2,}\\.?)' +
    ')' +
    // port number (optional)
    '(?::\\d{2,5})?' +
    // resource path (optional)
    '(?:[/?#]\\S*)?' +
    '$', 'i'
  )
  return !!reWebURL.test(input)
}

function validateURLInput () {
  const isValid = isValidUrl(this.value)
  if (isValid === false) {
    this.setCustomValidity('Please provide a valid URL')
    htmx.find('#createRedirectForm').reportValidity()
  } else {
    this.setCustomValidity('')
  }
}
window.validateURLInput = validateURLInput

/**
  * Shows a toast message to the user
  * @param {boolean} success - The type of the message (success, error, warning, info)
  * @param {string} message - The message to show
  * @returns {void}
  */
function toast (success, message) {
  const toast = document.createElement('div')
  toast.classList.add('fixed', 'bottom-0', 'right-0', 'm-4', 'p-4', 'text-white', 'rounded-lg', 'shadow-lg')
  if (success) {
    toast.classList.add('bg-green-500')
  } else {
    toast.classList.add('bg-red-500')
  }
  toast.innerText = message
  document.body.appendChild(toast)
  setTimeout(() => {
    toast.remove()
  }, 3000)
}

function copyRedirectKeyToClipboard (element, event) {
  // make sure the event is related to the api endpoint we are interested in
  if (event.detail.pathInfo.requestPath !== '/api/redirect') {
    return
  }

  if (event.detail.successful === true) {
    // copy the redirect key from the textbox into the clipboard
    try {
      const text = element.querySelector('#redirectKeyInput').value
      navigator.clipboard.writeText(text)
    } catch (err) {
      console.error('Failed to copy: ', err)
    }
    toast(true, 'Shortend URL copied to clipboard')

    // clear the URL input field and refresh the redirect key input
    element.querySelector('input[name="url"]').value = ''
    htmx.trigger('#randomizeButton', 'click')
  } else {
    toast(false, 'Failed to create redirect')
  }
}
window.copyRedirectKeyToClipboard = copyRedirectKeyToClipboard
