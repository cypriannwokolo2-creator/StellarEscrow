import {
  validateFeedback,
  hasErrors,
  createFeedbackEntry,
} from './feedbackSchema';
import { submit, getAnalytics, _clearStore } from './feedbackStore';

beforeEach(() => _clearStore());

// ---------------------------------------------------------------------------
// validateFeedback
// ---------------------------------------------------------------------------

describe('validateFeedback — rating type', () => {
  it('requires a rating', () => {
    const errs = validateFeedback('rating', 'good', undefined);
    expect(errs.rating).toBe('Please select a rating');
  });

  it('rejects out-of-range rating', () => {
    const errs = validateFeedback('rating', 'good', 6 as any);
    expect(errs.rating).toBeTruthy();
  });

  it('accepts valid rating + message', () => {
    const errs = validateFeedback('rating', 'great experience', 5);
    expect(hasErrors(errs)).toBe(false);
  });
});

describe('validateFeedback — message', () => {
  it('rejects empty message', () => {
    const errs = validateFeedback('bug', '   ');
    expect(errs.message).toBe('Message is required');
  });

  it('rejects message over 2000 chars', () => {
    const errs = validateFeedback('suggestion', 'a'.repeat(2001));
    expect(errs.message).toMatch(/2000/);
  });

  it('rejects HTML/script injection', () => {
    const errs = validateFeedback('bug', '<script>alert(1)</script>');
    expect(errs.message).toBe('Message contains invalid characters');
  });

  it('accepts a valid message', () => {
    const errs = validateFeedback('suggestion', 'Add dark mode please');
    expect(hasErrors(errs)).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// createFeedbackEntry
// ---------------------------------------------------------------------------

describe('createFeedbackEntry', () => {
  it('creates an entry with a unique id and ISO timestamp', () => {
    const entry = createFeedbackEntry('suggestion', 'Add dark mode');
    expect(entry.id).toMatch(/^fb_/);
    expect(entry.submittedAt).toMatch(/^\d{4}-\d{2}-\d{2}T/);
    expect(entry.type).toBe('suggestion');
    expect(entry.message).toBe('Add dark mode');
  });

  it('trims whitespace from message', () => {
    const entry = createFeedbackEntry('bug', '  crash on fund  ');
    expect(entry.message).toBe('crash on fund');
  });

  it('attaches bug metadata when provided', () => {
    const meta = { cliVersion: '1.2.3', os: 'linux', lastCommand: 'fund 42' };
    const entry = createFeedbackEntry('bug', 'crash', undefined, meta);
    expect(entry.metadata).toEqual(meta);
  });
});

// ---------------------------------------------------------------------------
// feedbackStore + getAnalytics
// ---------------------------------------------------------------------------

describe('getAnalytics', () => {
  it('returns zeros for empty store', () => {
    const a = getAnalytics();
    expect(a.totalCount).toBe(0);
    expect(a.averageRating).toBeNull();
    expect(a.bugCount).toBe(0);
  });

  it('computes average rating correctly', () => {
    submit(createFeedbackEntry('rating', 'ok', 4));
    submit(createFeedbackEntry('rating', 'great', 5));
    const a = getAnalytics();
    expect(a.averageRating).toBe(4.5);
    expect(a.ratingCount).toBe(2);
  });

  it('counts bugs and suggestions separately', () => {
    submit(createFeedbackEntry('bug', 'crash'));
    submit(createFeedbackEntry('bug', 'freeze'));
    submit(createFeedbackEntry('suggestion', 'dark mode'));
    const a = getAnalytics();
    expect(a.bugCount).toBe(2);
    expect(a.suggestionCount).toBe(1);
    expect(a.totalCount).toBe(3);
  });

  it('builds rating distribution', () => {
    submit(createFeedbackEntry('rating', '', 5));
    submit(createFeedbackEntry('rating', '', 5));
    submit(createFeedbackEntry('rating', '', 3));
    const { ratingDistribution } = getAnalytics();
    expect(ratingDistribution[5]).toBe(2);
    expect(ratingDistribution[3]).toBe(1);
    expect(ratingDistribution[1]).toBe(0);
  });
});
