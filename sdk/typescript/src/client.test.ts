import { Client } from './client';
import { SdkError } from './errors';

describe('Client', () => {
  describe('builder', () => {
    it('should create client with builder pattern', async () => {
      const client = await Client.builder()
        .name('Alice')
        .listenPort(8080)
        .build();

      expect(client.getIdentity().username).toBe('Alice');
      expect(client.getConfig().network.listenPort).toBe(8080);
    });

    it('should use default configuration', async () => {
      const client = await Client.builder()
        .name('Bob')
        .build();

      const config = client.getConfig();
      expect(config.name).toBe('Bob');
      expect(config.encryptionEnabled).toBe(true);
      expect(config.storage.maxSizeMb).toBe(1000);
    });
  });

  describe('connect/disconnect', () => {
    it('should connect and disconnect', async () => {
      const client = await Client.builder()
        .name('Charlie')
        .build();

      expect(client.isConnected()).toBe(false);

      await client.connect();
      expect(client.isConnected()).toBe(true);

      await client.disconnect();
      expect(client.isConnected()).toBe(false);
    });

    it('should throw error on double connect', async () => {
      const client = await Client.builder()
        .name('Dave')
        .build();

      await client.connect();
      await expect(client.connect()).rejects.toThrow(SdkError);
    });

    it('should not throw on double disconnect', async () => {
      const client = await Client.builder()
        .name('Eve')
        .build();

      await client.disconnect();
      await expect(client.disconnect()).resolves.not.toThrow();
    });
  });

  describe('messaging', () => {
    it('should throw error when sending without connection', async () => {
      const client = await Client.builder()
        .name('Frank')
        .build();

      await expect(client.sendMessage('Hello')).rejects.toThrow(SdkError);
    });

    it('should send and receive messages when connected', async () => {
      const client = await Client.builder()
        .name('Grace')
        .build();

      await client.connect();
      await client.sendMessage('Hello, dchat!');

      const messages = await client.receiveMessages();
      expect(messages).toHaveLength(1);
      expect(messages[0].content).toEqual({ type: 'Text', text: 'Hello, dchat!' });
    });

    it('should throw error when receiving without connection', async () => {
      const client = await Client.builder()
        .name('Heidi')
        .build();

      await expect(client.receiveMessages()).rejects.toThrow(SdkError);
    });
  });

  describe('identity', () => {
    it('should generate unique identity', async () => {
      const client1 = await Client.builder().name('Ivan').build();
      const client2 = await Client.builder().name('Judy').build();

      expect(client1.getIdentity().userId).not.toBe(client2.getIdentity().userId);
    });

    it('should include username in identity', async () => {
      const client = await Client.builder().name('Mallory').build();
      expect(client.getIdentity().username).toBe('Mallory');
    });
  });
});
