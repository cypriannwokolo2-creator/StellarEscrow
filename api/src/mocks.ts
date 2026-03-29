import { tradeHandlers } from './mocks/handlers/trades';
import { eventHandlers } from './mocks/handlers/events';
import { blockchainHandlers } from './mocks/handlers/blockchain';
export { resetMockData } from './mocks/data';

export const handlers = [
  ...tradeHandlers,
  ...eventHandlers,
  ...blockchainHandlers,
];
