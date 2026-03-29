import React from 'react';

interface StarRatingProps {
  value: number | undefined;
  onChange: (rating: number) => void;
  error?: string;
}

const LABELS: Record<number, string> = {
  1: 'Very dissatisfied',
  2: 'Dissatisfied',
  3: 'Neutral',
  4: 'Satisfied',
  5: 'Very satisfied',
};

export const StarRating: React.FC<StarRatingProps> = ({ value, onChange, error }) => {
  const [hovered, setHovered] = React.useState<number | null>(null);
  const id = React.useId();

  return (
    <fieldset className="star-rating-fieldset">
      <legend className="input-label">
        Satisfaction Rating <span aria-hidden="true">*</span>
      </legend>
      <div className="star-rating" role="group" aria-label="Satisfaction rating">
        {[1, 2, 3, 4, 5].map((star) => {
          const filled = (hovered ?? value ?? 0) >= star;
          return (
            <button
              key={star}
              type="button"
              className={`star-btn${filled ? ' star-btn--filled' : ''}`}
              aria-label={`${star} star${star > 1 ? 's' : ''} — ${LABELS[star]}`}
              aria-pressed={value === star}
              onClick={() => onChange(star)}
              onMouseEnter={() => setHovered(star)}
              onMouseLeave={() => setHovered(null)}
            >
              ★
            </button>
          );
        })}
      </div>
      {(hovered ?? value) && (
        <span className="star-label" aria-live="polite">
          {LABELS[hovered ?? value!]}
        </span>
      )}
      {error && (
        <span id={`${id}-error`} className="input-error-text input-error-text--visible" role="alert">
          {error}
        </span>
      )}
    </fieldset>
  );
};
