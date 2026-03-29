/**
 * In-memory feedback store with basic analytics aggregation.
 * Replace `submit` with a real API call in production.
 */

import { type FeedbackEntry, type FeedbackRating } from './feedbackSchema';

export interface FeedbackAnalytics {
  totalCount: number;
  averageRating: number | null;
  bugCount: number;
  suggestionCount: number;
  ratingCount: number;
  ratingDistribution: Record<FeedbackRating, number>;
}

// In-memory store (replace with API/DB in production)
const store: FeedbackEntry[] = [];

export function submit(entry: FeedbackEntry): void {
  store.push(entry);
}

export function getAll(): readonly FeedbackEntry[] {
  return store;
}

export function getAnalytics(): FeedbackAnalytics {
  const ratings = store
    .filter((e) => e.type === 'rating' && e.rating !== undefined)
    .map((e) => e.rating as FeedbackRating);

  const distribution: Record<FeedbackRating, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0 };
  for (const r of ratings) distribution[r]++;

  return {
    totalCount: store.length,
    averageRating:
      ratings.length > 0
        ? Math.round((ratings.reduce((a, b) => a + b, 0) / ratings.length) * 10) / 10
        : null,
    bugCount: store.filter((e) => e.type === 'bug').length,
    suggestionCount: store.filter((e) => e.type === 'suggestion').length,
    ratingCount: ratings.length,
    ratingDistribution: distribution,
  };
}

/** Exposed for testing only */
export function _clearStore(): void {
  store.length = 0;
}
