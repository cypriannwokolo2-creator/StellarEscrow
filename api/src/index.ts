import { ApiClient } from './client';
import { TradesApi, EventsApi, BlockchainApi } from './resources';
import { ApiClientConfig } from './types';
import { loadConfig, ApiConfig } from './config';

export class EscrowApi {
  private client: ApiClient;
  public trades: TradesApi;
  public events: EventsApi;
  public blockchain: BlockchainApi;

  constructor(config: ApiClientConfig) {
    this.client = new ApiClient(config);
    this.trades = new TradesApi(this.client);
    this.events = new EventsApi(this.client);
    this.blockchain = new BlockchainApi(this.client);
  }

  addAuthToken(token: string) {
    this.client.addRequestInterceptor((config) => {
      config.headers.Authorization = `Bearer ${token}`;
      return config;
    });
  }

  addErrorHandler(handler: (error: any) => void) {
    this.client.addErrorInterceptor(async (error) => {
      handler(error);
      throw error;
    });
  }

  addResponseLogger() {
    this.client.addResponseInterceptor((response) => {
      console.log(`[API] ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status}`);
      return response;
    });
  }
}

export const createApi = (baseURL: string, mockEnabled = false): EscrowApi => {
  const cfg = loadConfig({ baseUrl: baseURL, mockEnabled });
  return new EscrowApi({
    baseURL: cfg.baseUrl,
    timeout: cfg.timeoutMs,
    mockEnabled: cfg.mockEnabled,
    retryConfig: {
      maxRetries: cfg.retryMax,
      delayMs: cfg.retryDelayMs,
      backoffMultiplier: cfg.retryBackoffMultiplier,
    },
  });
};

export type { ApiConfig };
