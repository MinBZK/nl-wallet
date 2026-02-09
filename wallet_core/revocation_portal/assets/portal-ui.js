import { formatDeletionCode, calculateCursorPosition, validateDeletionCode } from './portal.js';

(function() {
  const input = document.getElementById('deletion-code');
  const errorDisplay = document.getElementById('error-message');
  if (!input || !errorDisplay) return;

  const validate = (showError = false) => {
    const { rawValue } = formatDeletionCode(input.value);
    const requiredMsg = input.dataset.requiredMessage;
    const lengthMsg = input.dataset.validationMessage;

    const validationKey = validateDeletionCode(rawValue);
    let activeMsg = "";

    if (validationKey === 'required') {
      activeMsg = requiredMsg;
    } else if (validationKey === 'invalid_length') {
      activeMsg = lengthMsg;
    }

    input.setCustomValidity(activeMsg);

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
    const oldPos = e.target.selectionStart;
    const oldVal = e.target.value;

    const { formatted, rawValue, addTrailingHyphen } = formatDeletionCode(oldVal);

    e.target.value = formatted;

    const newPos = calculateCursorPosition(oldVal, oldPos, formatted, rawValue, addTrailingHyphen);
    e.target.setSelectionRange(newPos, newPos);

    validate(rawValue.length > 18 || errorDisplay.classList.contains('visible'));
  });

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Backspace') {
      const { selectionStart, selectionEnd, value } = e.target;

      // If it's a simple cursor (no selection) at the very end
      // and the last character is a hyphen:
      if (selectionStart === selectionEnd && selectionStart === value.length && value.endsWith('-')) {
        e.preventDefault();
        // Slice off the hyphen AND the character before it
        const newValue = value.slice(0, -2);
        const { formatted, rawValue, addTrailingHyphen } = formatDeletionCode(newValue);

        e.target.value = formatted;

        // Calculate position for the new state
        const newPos = calculateCursorPosition(newValue, newValue.length, formatted, rawValue, addTrailingHyphen);
        e.target.setSelectionRange(newPos, newPos);

        validate(rawValue.length > 18 || errorDisplay.classList.contains('visible'));
      }
    }
  });

  input.addEventListener('invalid', (e) => {
    e.preventDefault();
    validate(true);
  });

  validate(false);
})();
