import { describe, it, expect } from 'vitest';
import { formatDeletionCode, calculateCursorPosition, validateDeletionCode } from '../portal.js';

describe('formatDeletionCode', () => {
  it('returns empty string for empty input', () => {
    const { formatted, rawValue } = formatDeletionCode('');
    expect(formatted).toBe('');
    expect(rawValue).toBe('');
  });

  it('converts lowercase to uppercase', () => {
    const { formatted } = formatDeletionCode('abcd');
    expect(formatted).toBe('ABCD-');
  });

  it('replaces confusable characters I and L with 1', () => {
    const { rawValue } = formatDeletionCode('ILIL');
    expect(rawValue).toBe('1111');
  });

  it('replaces confusable character O with 0', () => {
    const { rawValue } = formatDeletionCode('OOOO');
    expect(rawValue).toBe('0000');
  });

  it('strips invalid characters', () => {
    const { rawValue } = formatDeletionCode('A!B@C#D$');
    expect(rawValue).toBe('ABCD');
  });

  it('inserts hyphens every 4 characters', () => {
    const { formatted } = formatDeletionCode('C20CKF0RD32B');
    expect(formatted).toBe('C20C-KF0R-D32B-');
  });

  it('adds trailing hyphen at group boundary of 4', () => {
    const { formatted, addTrailingHyphen } = formatDeletionCode('C20C');
    expect(formatted).toBe('C20C-');
    expect(addTrailingHyphen).toBe(true);
  });

  it('adds trailing hyphen at group boundary of 8', () => {
    const { formatted, addTrailingHyphen } = formatDeletionCode('C20CKF0R');
    expect(formatted).toBe('C20C-KF0R-');
    expect(addTrailingHyphen).toBe(true);
  });

  it('does not add trailing hyphen at 18 characters (complete code)', () => {
    const { formatted, addTrailingHyphen } = formatDeletionCode('C20CKF0RD32BA5E32X');
    expect(formatted).toBe('C20C-KF0R-D32B-A5E3-2X');
    expect(addTrailingHyphen).toBe(false);
  });

  it('does not add trailing hyphen for non-boundary lengths', () => {
    const { formatted, addTrailingHyphen } = formatDeletionCode('C20');
    expect(formatted).toBe('C20');
    expect(addTrailingHyphen).toBe(false);
  });

  it('allows more than 18 characters (no truncation)', () => {
    const { rawValue } = formatDeletionCode('C20CKF0RD32BA5E32XYZZ');
    expect(rawValue).toBe('C20CKF0RD32BA5E32XYZZ');
  });

  it('formats input that already contains hyphens', () => {
    const { formatted } = formatDeletionCode('C20C-KF0R');
    expect(formatted).toBe('C20C-KF0R-');
  });
});

describe('calculateCursorPosition', () => {
  it('places cursor at end when typing at end', () => {
    const pos = calculateCursorPosition('C20', 3, 'C20C', 'C20C', false);
    expect(pos).toBe(3);
  });

  it('moves cursor past trailing hyphen when at end', () => {
    const pos = calculateCursorPosition('C20C', 4, 'C20C-', 'C20C', true);
    expect(pos).toBe(5);
  });

  it('preserves cursor in middle when typing', () => {
    // User typed a char at position 2 in "C2-0CKF0R" → old value has cursor at pos 3
    // After formatting: "C2A0-CKF0-R"
    const pos = calculateCursorPosition('C2A-0CKF0R', 3, 'C2A0-CKF0-R', 'C2A0CKF0R', false);
    expect(pos).toBe(3);
  });

  it('accounts for hyphen insertion when cursor crosses boundary', () => {
    // User typed 4th char: "C20C" → cursor at 4
    // Formatted: "C20C-" with trailing hyphen
    const pos = calculateCursorPosition('C20C', 4, 'C20C-', 'C20C', true);
    expect(pos).toBe(5); // past the hyphen
  });

  it('keeps cursor at start when at position 0', () => {
    const pos = calculateCursorPosition('C20C-KF0R', 0, 'C20C-KF0R-', 'C20CKF0R', true);
    expect(pos).toBe(0);
  });
});

describe('validateDeletionCode', () => {
  it('returns required for empty input', () => {
    expect(validateDeletionCode('')).toBe('required');
  });

  it('returns invalid_length for too short input', () => {
    expect(validateDeletionCode('C20C')).toBe('invalid_length');
  });

  it('returns invalid_length for too long input', () => {
    expect(validateDeletionCode('C20CKF0RD32BA5E32XYZ')).toBe('invalid_length');
  });

  it('returns invalid_length for 17 characters', () => {
    expect(validateDeletionCode('C20CKF0RD32BA5E32')).toBe('invalid_length');
  });

  it('returns null for exactly 18 characters', () => {
    expect(validateDeletionCode('C20CKF0RD32BA5E32X')).toBeNull();
  });
});
