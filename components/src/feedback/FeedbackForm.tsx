import React from 'react';
import { Button } from '../base/Button';
import { Alert } from '../base/Alert';
import { StarRating } from './StarRating';
import {
  validateFeedback,
  hasErrors,
  createFeedbackEntry,
  type FeedbackType,
  type FeedbackRating,
  type BugMetadata,
} from './feedbackSchema';
import { submit } from './feedbackStore';
import './FeedbackForm.css';

export interface FeedbackFormProps {
  /** Called after successful submission with the stored entry id */
  onSuccess?: (id: string) => void;
  /** Pre-select a tab (default: 'rating') */
  defaultType?: FeedbackType;
  /** Bug metadata injected by the host (CLI version, OS, last command) */
  bugMetadata?: BugMetadata;
}

const TAB_LABELS: Record<FeedbackType, string> = {
  rating: '⭐ Rate',
  bug: '🐛 Report Bug',
  suggestion: '💡 Suggest',
};

export const FeedbackForm: React.FC<FeedbackFormProps> = ({
  onSuccess,
  defaultType = 'rating',
  bugMetadata,
}) => {
  const [type, setType] = React.useState<FeedbackType>(defaultType);
  const [rating, setRating] = React.useState<FeedbackRating | undefined>();
  const [message, setMessage] = React.useState('');
  const [errors, setErrors] = React.useState<{ rating?: string; message?: string }>({});
  const [submitted, setSubmitted] = React.useState(false);
  const messageId = React.useId();

  const placeholders: Record<FeedbackType, string> = {
    rating: 'Tell us more about your experience (optional)…',
    bug: 'Describe what happened and the steps to reproduce…',
    suggestion: 'Describe your feature request or improvement idea…',
  };

  function handleTabChange(next: FeedbackType) {
    setType(next);
    setErrors({});
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const errs = validateFeedback(type, message, rating);
    if (hasErrors(errs)) {
      setErrors(errs);
      return;
    }
    const entry = createFeedbackEntry(
      type,
      message,
      type === 'rating' ? rating : undefined,
      type === 'bug' ? bugMetadata : undefined
    );
    submit(entry);
    setSubmitted(true);
    onSuccess?.(entry.id);
  }

  if (submitted) {
    return (
      <div className="feedback-form" role="status">
        <Alert type="success" title="Thank you!">
          Your feedback has been recorded.
        </Alert>
      </div>
    );
  }

  return (
    <form className="feedback-form" onSubmit={handleSubmit} noValidate aria-label="Feedback form">
      {/* Tab switcher */}
      <div className="feedback-tabs" role="tablist" aria-label="Feedback type">
        {(Object.keys(TAB_LABELS) as FeedbackType[]).map((t) => (
          <button
            key={t}
            type="button"
            role="tab"
            aria-selected={type === t}
            className={`feedback-tab${type === t ? ' feedback-tab--active' : ''}`}
            onClick={() => handleTabChange(t)}
          >
            {TAB_LABELS[t]}
          </button>
        ))}
      </div>

      <div className="feedback-body">
        {/* Star rating — only for 'rating' tab */}
        {type === 'rating' && (
          <StarRating value={rating} onChange={(v) => setRating(v as FeedbackRating)} error={errors.rating} />
        )}

        {/* Bug metadata preview */}
        {type === 'bug' && bugMetadata && (
          <div className="feedback-metadata" aria-label="Captured environment info">
            <p className="feedback-metadata__title">Captured automatically:</p>
            <dl className="feedback-metadata__list">
              <dt>CLI Version</dt><dd>{bugMetadata.cliVersion}</dd>
              <dt>OS</dt><dd>{bugMetadata.os}</dd>
              <dt>Last Command</dt><dd><code>{bugMetadata.lastCommand}</code></dd>
            </dl>
          </div>
        )}

        {/* Message textarea */}
        <div className={`input-wrapper${errors.message ? ' input-wrapper--error' : ''}`}>
          <label htmlFor={messageId} className="input-label">
            {type === 'bug' ? 'Bug Description' : type === 'suggestion' ? 'Your Suggestion' : 'Comments'}
            {type !== 'rating' && <span aria-hidden="true"> *</span>}
          </label>
          <textarea
            id={messageId}
            className={`feedback-textarea input${errors.message ? ' input-error' : ''}`}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder={placeholders[type]}
            rows={4}
            maxLength={2000}
            aria-invalid={errors.message ? true : undefined}
            aria-describedby={errors.message ? `${messageId}-error` : undefined}
          />
          <span className="feedback-char-count" aria-live="polite">
            {message.length}/2000
          </span>
          {errors.message && (
            <span
              id={`${messageId}-error`}
              className="input-error-text input-error-text--visible"
              role="alert"
            >
              {errors.message}
            </span>
          )}
        </div>

        <Button type="submit" variant="primary">
          Submit Feedback
        </Button>
      </div>
    </form>
  );
};
