/**
 * Feedback system — shared types and validation schema.
 * Pure functions, no external dependencies.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type FeedbackType = 'rating' | 'bug' | 'suggestion';

export type FeedbackRating = 1 | 2 | 3 | 4 | 5;

export interface BugMetadata {
  cliVersion: string;
  os: string;
  lastCommand: string;
}

export interface FeedbackEntry {
  id: string;
  type: FeedbackType;
  rating?: FeedbackRating;
  message: string;
  metadata?: BugMetadata;
  /** ISO 8601 timestamp */
  submittedAt: string;
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

export interface FeedbackErrors {
  rating?: string;
  message?: string;
}

const MAX_MESSAGE_LENGTH = 2000;
/** Basic XSS / injection guard: reject strings that look like HTML/script tags */
const UNSAFE_PATTERN = /<[^>]*>|javascript:/i;

export function validateFeedback(
  type: FeedbackType,
  message: string,
  rating?: number
): FeedbackErrors {
  const errors: FeedbackErrors = {};

  if (type === 'rating') {
    if (rating === undefined || rating === null) {
      errors.rating = 'Please select a rating';
    } else if (!Number.isInteger(rating) || rating < 1 || rating > 5) {
      errors.rating = 'Rating must be between 1 and 5';
    }
  }

  const trimmed = message.trim();
  if (type !== 'rating' && trimmed.length === 0) {
    errors.message = 'Message is required';
  } else if (trimmed.length > MAX_MESSAGE_LENGTH) {
    errors.message = `Message must be ${MAX_MESSAGE_LENGTH} characters or fewer`;
  } else if (UNSAFE_PATTERN.test(trimmed)) {
    errors.message = 'Message contains invalid characters';
  }

  return errors;
}

export function hasErrors(errors: FeedbackErrors): boolean {
  return Object.keys(errors).length > 0;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

export function createFeedbackEntry(
  type: FeedbackType,
  message: string,
  rating?: FeedbackRating,
  metadata?: BugMetadata
): FeedbackEntry {
  return {
    id: `fb_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
    type,
    rating,
    message: message.trim(),
    metadata,
    submittedAt: new Date().toISOString(),
  };
}
