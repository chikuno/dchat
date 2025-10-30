import {
  defaultClientConfig,
  defaultStorageConfig,
  defaultNetworkConfig,
  defaultRelayConfig,
} from './config';

describe('Configuration', () => {
  describe('defaultStorageConfig', () => {
    it('should return default storage config', () => {
      const config = defaultStorageConfig();

      expect(config.dataDir).toBe('./dchat_data');
      expect(config.maxSizeMb).toBe(1000);
      expect(config.cacheEnabled).toBe(true);
    });
  });

  describe('defaultNetworkConfig', () => {
    it('should return default network config', () => {
      const config = defaultNetworkConfig();

      expect(config.bootstrapPeers).toEqual([]);
      expect(config.listenPort).toBe(0);
      expect(config.maxConnections).toBe(50);
      expect(config.timeoutMs).toBe(30000);
    });
  });

  describe('defaultClientConfig', () => {
    it('should return default client config', () => {
      const config = defaultClientConfig('TestUser');

      expect(config.name).toBe('TestUser');
      expect(config.encryptionEnabled).toBe(true);
      expect(config.storage).toBeDefined();
      expect(config.network).toBeDefined();
    });

    it('should use default name if not provided', () => {
      const config = defaultClientConfig();
      expect(config.name).toBe('User');
    });
  });

  describe('defaultRelayConfig', () => {
    it('should return default relay config', () => {
      const config = defaultRelayConfig();

      expect(config.name).toBe('dchat-relay');
      expect(config.listenAddr).toBe('0.0.0.0');
      expect(config.listenPort).toBe(9000);
      expect(config.stakingEnabled).toBe(false);
      expect(config.minUptimePercent).toBe(95.0);
    });
  });
});
