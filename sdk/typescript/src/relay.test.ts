import { RelayNode } from './relay';
import { defaultRelayConfig } from './config';
import { SdkError } from './errors';

describe('RelayNode', () => {
  describe('creation', () => {
    it('should create relay with default config', () => {
      const relay = RelayNode.create();
      const config = relay.getConfig();

      expect(config.listenPort).toBe(9000);
      expect(config.minUptimePercent).toBe(95.0);
    });

    it('should create relay with custom config', () => {
      const config = {
        ...defaultRelayConfig(),
        name: 'MyRelay',
        listenPort: 8080,
      };

      const relay = RelayNode.withConfig(config);
      expect(relay.getConfig().name).toBe('MyRelay');
      expect(relay.getConfig().listenPort).toBe(8080);
    });
  });

  describe('lifecycle', () => {
    it('should start and stop', async () => {
      const relay = RelayNode.create();

      expect(relay.isRunning()).toBe(false);

      await relay.start();
      expect(relay.isRunning()).toBe(true);

      await relay.stop();
      expect(relay.isRunning()).toBe(false);
    });

    it('should throw error on double start', async () => {
      const relay = RelayNode.create();

      await relay.start();
      await expect(relay.start()).rejects.toThrow(SdkError);
    });

    it('should not throw on double stop', async () => {
      const relay = RelayNode.create();

      await relay.stop();
      await expect(relay.stop()).resolves.not.toThrow();
    });
  });

  describe('statistics', () => {
    it('should return stats', async () => {
      const relay = RelayNode.create();
      await relay.start();

      const stats = await relay.getStats();

      expect(stats.connectedPeers).toBe(0);
      expect(stats.messagesRelayed).toBe(0);
      expect(stats.uptimePercent).toBeGreaterThan(0);
      expect(stats.reputationScore).toBe(100);
    });

    it('should calculate uptime', async () => {
      const relay = RelayNode.create();
      await relay.start();

      await new Promise(resolve => setTimeout(resolve, 100));

      const stats = await relay.getStats();
      expect(stats.uptimePercent).toBeGreaterThan(0);
    });
  });
});
