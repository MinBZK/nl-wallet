(function() {
  const input = document.getElementById('deletion-code');
  const errorDisplay = document.getElementById('error-message');
  if (!input || !errorDisplay) return;

  // Disable native pattern validation since JS handles it with better UX
  input.removeAttribute('pattern');
  input.removeAttribute('title');

  const ALLOWED_REGEX = /[^0-9A-HJKMNP-TV-Z]/g;

  const validate = (showError = false) => {
    const val = input.value.replace(/-/g, '');
    const requiredMsg = input.dataset.requiredMessage;
    const lengthMsg = input.dataset.validationMessage;
    let activeMsg = "";

    if (val.length === 0) {
      activeMsg = requiredMsg;
    } else if (val.length !== 18) {
      activeMsg = lengthMsg;
    }

    // Update custom validity (blocks form submission)
    input.setCustomValidity(activeMsg);

    // Update our styled display
    if (showError && activeMsg) {
      errorDisplay.textContent = activeMsg;
      errorDisplay.classList.add('visible');
      input.classList.add('invalid');
    } else {
      errorDisplay.classList.remove('visible');
      input.classList.remove('invalid');
      errorDisplay.textContent = '';
    }
  };

  input.addEventListener('input', (e) => {
    let val = e.target.value.toUpperCase();
    val = val.replace(/[IL]/g, '1').replace(/O/g, '0');
    let rawValue = val.replace(ALLOWED_REGEX, '');
    const parts = rawValue.match(/.{1,4}/g);
    let formatted = parts ? parts.join('-') : rawValue;

    // Add trailing hyphen right after typing every 4th character
    if (rawValue.length > 0 && rawValue.length < 18 && rawValue.length % 4 === 0) {
      formatted += '-';
    }

    e.target.value = formatted;

    // Show error immediately if too long, otherwise follow existing logic
    const isTooLong = rawValue.length > 18;
    validate(isTooLong || errorDisplay.classList.contains('visible'));
  });

  // Show error when the browser flags the field as invalid (e.g. on submit)
  input.addEventListener('invalid', (e) => {
    e.preventDefault(); // Stop native tooltip
    validate(true);
  });

  // Initial check
  validate(false);
})();
