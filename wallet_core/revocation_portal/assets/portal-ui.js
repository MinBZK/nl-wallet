import { formatDeletionCode, calculateCursorPosition, validateDeletionCode } from './portal.js';

(function() {
  const input = document.getElementById('deletion-code');
  const errorDisplay = document.getElementById('error-message');
  if (!input || !errorDisplay) return;

  // Disable native pattern validation since JS handles it with better UX
  input.removeAttribute('pattern');
  input.removeAttribute('title');

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

  input.addEventListener('invalid', (e) => {
    e.preventDefault();
    validate(true);
  });

  validate(false);
})();
