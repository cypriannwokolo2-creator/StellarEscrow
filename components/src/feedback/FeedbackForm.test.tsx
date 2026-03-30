import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { FeedbackForm } from './FeedbackForm';
import { _clearStore, getAnalytics } from './feedbackStore';

beforeEach(() => _clearStore());

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

describe('FeedbackForm — rendering', () => {
  it('renders all three tabs', () => {
    render(<FeedbackForm />);
    expect(screen.getByRole('tab', { name: /rate/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /report bug/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /suggest/i })).toBeInTheDocument();
  });

  it('shows star buttons on the rating tab', () => {
    render(<FeedbackForm defaultType="rating" />);
    expect(screen.getByLabelText(/5 stars/i)).toBeInTheDocument();
  });

  it('shows bug metadata when provided', () => {
    render(
      <FeedbackForm
        defaultType="bug"
        bugMetadata={{ cliVersion: '1.0.0', os: 'linux', lastCommand: 'fund 1' }}
      />
    );
    expect(screen.getByText('1.0.0')).toBeInTheDocument();
    expect(screen.getByText('linux')).toBeInTheDocument();
    expect(screen.getByText('fund 1')).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

describe('FeedbackForm — validation', () => {
  it('shows rating error when submitting rating tab without a star', () => {
    render(<FeedbackForm defaultType="rating" />);
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(screen.getByText('Please select a rating')).toBeInTheDocument();
  });

  it('shows message error when submitting bug tab with empty message', () => {
    render(<FeedbackForm defaultType="bug" />);
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(screen.getByText('Message is required')).toBeInTheDocument();
  });

  it('does not submit when validation fails', () => {
    const onSuccess = jest.fn();
    render(<FeedbackForm defaultType="bug" onSuccess={onSuccess} />);
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(onSuccess).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Successful submission
// ---------------------------------------------------------------------------

describe('FeedbackForm — submission', () => {
  it('shows success message after valid rating submission', () => {
    render(<FeedbackForm defaultType="rating" />);
    fireEvent.click(screen.getByLabelText(/5 stars/i));
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(screen.getByText(/thank you/i)).toBeInTheDocument();
  });

  it('calls onSuccess with an entry id', () => {
    const onSuccess = jest.fn();
    render(<FeedbackForm defaultType="suggestion" onSuccess={onSuccess} />);
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'Add dark mode' } });
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(onSuccess).toHaveBeenCalledWith(expect.stringMatching(/^fb_/));
  });

  it('stores the entry in the feedback store', () => {
    render(<FeedbackForm defaultType="suggestion" />);
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'Add dark mode' } });
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(getAnalytics().suggestionCount).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// Tab switching
// ---------------------------------------------------------------------------

describe('FeedbackForm — tab switching', () => {
  it('switches to bug tab and hides star buttons', () => {
    render(<FeedbackForm defaultType="rating" />);
    fireEvent.click(screen.getByRole('tab', { name: /report bug/i }));
    expect(screen.queryByLabelText(/5 stars/i)).not.toBeInTheDocument();
  });

  it('clears errors when switching tabs', () => {
    render(<FeedbackForm defaultType="bug" />);
    fireEvent.click(screen.getByRole('button', { name: /submit feedback/i }));
    expect(screen.getByText('Message is required')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab', { name: /suggest/i }));
    expect(screen.queryByText('Message is required')).not.toBeInTheDocument();
  });
});
