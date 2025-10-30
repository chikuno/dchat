/**
 * dchat Client implementation
 */

import { v4 as uuidv4 } from 'uuid';
import { ClientConfig, defaultClientConfig } from './config';
import { SdkError } from './errors';
import { Identity, Message, MessageStatus } from './types';

export class Client {
  private config: ClientConfig;
  private identity: Identity;
  private connected: boolean = false;
  private messages: Message[] = [];

  private constructor(config: ClientConfig, identity: Identity) {
    this.config = config;
    this.identity = identity;
  }

  /**
   * Create a new client builder
   */
  static builder(): ClientBuilder {
    return new ClientBuilder();
  }

  /**
   * Create a client with custom configuration
   */
  static async create(config: ClientConfig): Promise<Client> {
    // Generate identity
    const identity: Identity = {
      userId: uuidv4(),
      username: config.name,
      publicKey: generateKeyPair(), // Placeholder
      reputation: 0,
      createdAt: new Date(),
      verified: false,
      badges: [],
    };

    return new Client(config, identity);
  }

  /**
   * Connect to the dchat network
   */
  async connect(): Promise<void> {
    if (this.connected) {
      throw SdkError.alreadyConnected();
    }

    // TODO: Implement network connection
    this.connected = true;
  }

  /**
   * Disconnect from the network
   */
  async disconnect(): Promise<void> {
    if (!this.connected) {
      return;
    }

    // TODO: Implement network disconnection
    this.connected = false;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Send a text message
   */
  async sendMessage(text: string): Promise<void> {
    if (!this.connected) {
      throw SdkError.notConnected();
    }

    const message: Message = {
      id: uuidv4(),
      senderId: this.identity.userId,
      content: { type: 'Text', text },
      encryptedPayload: new Uint8Array(0),
      timestamp: new Date(),
      status: MessageStatus.Created,
      size: text.length,
    };

    // TODO: Send to network
    
    // Store locally
    this.messages.push(message);
  }

  /**
   * Receive messages
   */
  async receiveMessages(): Promise<Message[]> {
    if (!this.connected) {
      throw SdkError.notConnected();
    }

    // TODO: Fetch from network
    
    return [...this.messages];
  }

  /**
   * Get the client's identity
   */
  getIdentity(): Identity {
    return { ...this.identity };
  }

  /**
   * Get the client's configuration
   */
  getConfig(): ClientConfig {
    return { ...this.config };
  }
}

/**
 * Builder for creating a Client
 */
export class ClientBuilder {
  private config: ClientConfig;

  constructor() {
    this.config = defaultClientConfig();
  }

  /**
   * Set the user's display name
   */
  name(name: string): this {
    this.config.name = name;
    return this;
  }

  /**
   * Set the storage directory
   */
  dataDir(path: string): this {
    this.config.storage.dataDir = path;
    return this;
  }

  /**
   * Set bootstrap peers
   */
  bootstrapPeers(peers: string[]): this {
    this.config.network.bootstrapPeers = peers;
    return this;
  }

  /**
   * Set the listen port
   */
  listenPort(port: number): this {
    this.config.network.listenPort = port;
    return this;
  }

  /**
   * Enable or disable encryption
   */
  encryption(enabled: boolean): this {
    this.config.encryptionEnabled = enabled;
    return this;
  }

  /**
   * Build the client
   */
  async build(): Promise<Client> {
    return Client.create(this.config);
  }
}

// Placeholder for key generation
function generateKeyPair(): string {
  return uuidv4(); // In real implementation, generate actual Ed25519 keys
}
