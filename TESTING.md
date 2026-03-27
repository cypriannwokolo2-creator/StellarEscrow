# Testing Guidelines

## Overview

This project uses a three-tier testing strategy:

| Layer | Tool | Location | Purpose |
|-------|------|----------|---------|
| Unit | Jest + ts-jest | `*/src/**/*.test.ts(x)` | Functions, hooks, slices |
| Integration | Jest + React Testing Library / MSW | `components/src/**/*.test.tsx`, `api/src/**/*.integration.test.ts` | Component and API behaviour |
| Contract | Jest + runtime validators | `api/src/**/*.contract.test.ts` | Endpoint method/path/shape guarantees |
| Load | Jest + concurrent request harness | `api/src/**/*.load.test.ts` | API concurrency smoke coverage |
| Stress | Jest + performance harness | `api/src/**/*.stress.test.ts` | Burst traffic, retry pressure, and degraded upstream behaviour |
| Benchmark | Jest + performance harness | `api/src/**/*.benchmark.test.ts` | Endpoint and workflow latency baselines |
| Scalability | Jest + performance harness | `api/src/**/*.scalability.test.ts` | Throughput and latency trends across concurrency levels |
| Monitoring | Jest + performance harness | `api/src/**/*.monitoring.test.ts` | Threshold alerts, error-rate visibility, and per-operation telemetry |
| Security | Jest + scenario harness | `security/src/security.assessment.test.ts` | Penetration tests, vulnerability scans, compliance, monitoring |
| E2E | Cypress | `components/cypress/e2e/` | Full user flows |

## Running Tests

```bash
# All unit/integration tests across all packages
npm test

# With coverage reports
npm run test:coverage

# API package suites
npm run test:integration --workspace=api
npm run test:contract --workspace=api
npm run test:load --workspace=api
npm run test:stress --workspace=api
npm run test:benchmark --workspace=api
npm run test:scalability --workspace=api
npm run test:monitoring --workspace=api
npm run test:performance --workspace=api
npm run test:docs --workspace=api

# Security assessment suite
npm run test:security
npm run test:security --workspace=security

# E2E tests (requires app running on localhost:3000)
npm run test:e2e

# Watch mode (inside a package)
cd components && npm run test:watch
```

## Coverage Thresholds

All packages enforce **70% minimum** on branches, functions, lines, and statements.
Coverage reports are written to `coverage/` in each package directory.

## Unit Tests

- One test file per source file, co-located: `Button.tsx` → `Button.test.tsx`
- Use `describe` to group related cases; use `it`/`test` for individual assertions
- Mock external dependencies with `jest.fn()` or `jest.mock()`
- Keep tests independent — no shared mutable state between tests

```ts
// Good
it('disables button when loading', () => {
  render(<Button loading>Save</Button>);
  expect(screen.getByText('Save')).toBeDisabled();
});
```

## Integration Tests (React Testing Library)

- Test behaviour, not implementation details
- Query by accessible roles/labels (`getByRole`, `getByLabelText`) over test IDs
- Use `userEvent` over `fireEvent` for realistic interactions
- Assert on what the user sees, not internal state

```ts
import userEvent from '@testing-library/user-event';

it('submits form with valid data', async () => {
  const onSubmit = jest.fn();
  render(<TradeForm onSubmit={onSubmit} />);
  await userEvent.type(screen.getByLabelText('Amount (USDC)'), '100');
  await userEvent.click(screen.getByRole('button', { name: 'Create Trade' }));
  expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ amount: '100' }));
});
```

## E2E Tests (Cypress)

- Located in `components/cypress/e2e/`
- Use `data-testid` attributes for selectors that are stable across refactors
- Each spec file covers one user flow (e.g., `trade-flow.cy.ts`, `dispute-flow.cy.ts`)
- Run against a locally running app: `npm run test:e2e:open` for interactive mode

```ts
it('creates a new trade', () => {
  cy.visit('/');
  cy.get('[data-testid="create-trade-btn"]').click();
  cy.get('[data-testid="amount"]').type('100');
  cy.get('[data-testid="submit-trade"]').click();
  cy.get('[data-testid="trade-status"]').should('contain', 'created');
});
```

## What to Test

**Do test:**
- Happy path and error/edge cases for business logic
- Component rendering with different prop combinations
- Form validation and submission
- Redux slice reducers and selectors
- Security utilities (sanitization, validation)
- Security attack scenarios, compliance controls, vulnerability baselines, and monitoring alerts

**Don't test:**
- Third-party library internals
- Implementation details (internal state, private methods)
- Styling/CSS classes unless they affect behaviour

## Adding New Tests

1. Create `<ComponentName>.test.tsx` alongside the source file
2. Import from `@testing-library/react` for React components
3. Run `npm run test:coverage` to verify thresholds are met
4. For new user flows, add a Cypress spec in `components/cypress/e2e/`
