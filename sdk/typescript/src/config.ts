/**
 * Configuration types for dchat SDK
 */

export interface StorageConfig {
  /** Path to data directory */
  dataDir: string;
  /** Maximum storage size in MB */
  maxSizeMb: number;
  /** Enable caching */
  cacheEnabled: boolean;
}

export interface NetworkConfig {
  /** Bootstrap peer addresses */
  bootstrapPeers: string[];
  /** Listen port (0 for random) */
  listenPort: number;
  /** Maximum concurrent connections */
  maxConnections: number;
  /** Connection timeout in milliseconds */
  timeoutMs: number;
}

export interface ClientConfig {
  /** User display name */
  name: string;
  /** Storage configuration */
  storage: StorageConfig;
  /** Network configuration */
  network: NetworkConfig;
  /** Enable end-to-end encryption */
  encryptionEnabled: boolean;
}

export interface RelayConfig {
  /** Node display name */
  name: string;
  /** Listen address */
  listenAddr: string;
  /** Listen port */
  listenPort: number;
  /** Enable staking rewards */
  stakingEnabled: boolean;
  /** Minimum uptime percentage for rewards */
  minUptimePercent: number;
}

/**
 * Create default storage configuration
 */
export function defaultStorageConfig(): StorageConfig {
  return {
    dataDir: './dchat_data',
    maxSizeMb: 1000,
    cacheEnabled: true,
  };
}

/**
 * Create default network configuration
 */
export function defaultNetworkConfig(): NetworkConfig {
  return {
    bootstrapPeers: [],
    listenPort: 0,
    maxConnections: 50,
    timeoutMs: 30000,
  };
}

/**
 * Create default client configuration
 */
export function defaultClientConfig(name: string = 'User'): ClientConfig {
  return {
    name,
    storage: defaultStorageConfig(),
    network: defaultNetworkConfig(),
    encryptionEnabled: true,
  };
}

/**
 * Create default relay configuration
 */
export function defaultRelayConfig(): RelayConfig {
  return {
    name: 'dchat-relay',
    listenAddr: '0.0.0.0',
    listenPort: 9000,
    stakingEnabled: false,
    minUptimePercent: 95.0,
  };
}
