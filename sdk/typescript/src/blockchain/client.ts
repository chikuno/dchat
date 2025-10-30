/**
 * Blockchain client for transaction submission and confirmation
 */

import { createHash } from 'crypto';
import {
  RegisterUserTx,
  SendDirectMessageTx,
  CreateChannelTx,
  PostToChannelTx,
  TransactionReceipt,
  ChannelVisibility,
} from './transaction';

export interface BlockchainConfig {
  rpcUrl: string;
  wsUrl?: string;
  confirmationBlocks: number;
  confirmationTimeout: number; // milliseconds
  maxRetries: number;
}

export class BlockchainClient {
  private config: BlockchainConfig;
  private transactionCache: Map<string, TransactionReceipt>;

  constructor(config: BlockchainConfig) {
    this.config = config;
    this.transactionCache = new Map();
  }

  static local(): BlockchainClient {
    return new BlockchainClient({
      rpcUrl: 'http://localhost:8545',
      wsUrl: 'ws://localhost:8546',
      confirmationBlocks: 6,
      confirmationTimeout: 300000, // 5 minutes
      maxRetries: 3,
    });
  }

  /**
   * Register a new user on-chain
   */
  async registerUser(
    userId: string,
    username: string,
    publicKey: string
  ): Promise<string> {
    const tx: RegisterUserTx = {
      userId,
      username,
      publicKey,
      timestamp: new Date(),
      initialReputation: 100,
    };

    return this.submitTransaction('register_user', tx);
  }

  /**
   * Send a direct message on-chain
   */
  async sendDirectMessage(
    messageId: string,
    senderId: string,
    recipientId: string,
    contentHash: string,
    payloadSize: number,
    relayNodeId?: string
  ): Promise<string> {
    const tx: SendDirectMessageTx = {
      messageId,
      senderId,
      recipientId,
      contentHash,
      payloadSize,
      timestamp: new Date(),
      relayNodeId,
    };

    return this.submitTransaction('send_direct_message', tx);
  }

  /**
   * Create a new channel on-chain
   */
  async createChannel(
    channelId: string,
    name: string,
    description: string,
    creatorId: string,
    visibility: ChannelVisibility = ChannelVisibility.Public,
    tokenRequirement?: string
  ): Promise<string> {
    const tx: CreateChannelTx = {
      channelId,
      name,
      description,
      creatorId,
      visibility,
      timestamp: new Date(),
      tokenRequirement,
    };

    return this.submitTransaction('create_channel', tx);
  }

  /**
   * Post a message to a channel on-chain
   */
  async postToChannel(
    messageId: string,
    channelId: string,
    senderId: string,
    contentHash: string,
    payloadSize: number
  ): Promise<string> {
    const tx: PostToChannelTx = {
      messageId,
      channelId,
      senderId,
      contentHash,
      payloadSize,
      timestamp: new Date(),
    };

    return this.submitTransaction('post_to_channel', tx);
  }

  /**
   * Wait for transaction confirmation
   */
  async waitForConfirmation(txId: string): Promise<TransactionReceipt> {
    // Check cache first
    const cached = this.transactionCache.get(txId);
    if (cached && (cached.success || cached.error)) {
      return cached;
    }

    const startTime = Date.now();
    const deadline = startTime + this.config.confirmationTimeout;

    while (Date.now() < deadline) {
      const receipt = await this.getTransactionReceipt(txId);
      if (receipt) {
        this.transactionCache.set(txId, receipt);

        if (receipt.success) {
          return receipt;
        } else if (receipt.error) {
          throw new Error(`Transaction failed: ${receipt.error}`);
        }
      }

      // Poll every 2 seconds
      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    throw new Error(`Transaction confirmation timed out after ${this.config.confirmationTimeout}ms`);
  }

  /**
   * Check if a transaction is confirmed
   */
  async isTransactionConfirmed(txId: string): Promise<boolean> {
    try {
      const receipt = await this.getTransactionReceipt(txId);
      return receipt?.success ?? false;
    } catch {
      return false;
    }
  }

  /**
   * Get transaction receipt
   */
  async getTransactionReceipt(txId: string): Promise<TransactionReceipt | null> {
    try {
      const response = await fetch(this.config.rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'eth_getTransactionReceipt',
          params: [txId],
          id: 1,
        }),
      });

      if (!response.ok) {
        return null;
      }

      const data = (await response.json()) as any;
      if (data.result) {
        return {
          txId: data.result.tx_id,
          txHash: data.result.tx_hash,
          success: data.result.success,
          blockHeight: data.result.block_height,
          blockHash: data.result.block_hash,
          timestamp: new Date(data.result.timestamp),
          error: data.result.error,
        };
      }

      return null;
    } catch {
      return null;
    }
  }

  /**
   * Get current block number
   */
  async getBlockNumber(): Promise<number> {
    const response = await fetch(this.config.rpcUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'eth_blockNumber',
        params: [],
        id: 1,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to get block number: ${response.status}`);
    }

    const data = (await response.json()) as any;
    return parseInt(data.result, 16);
  }

  /**
   * Subscribe to transaction confirmations via WebSocket
   */
  subscribeToConfirmations(
    txId: string,
    onConfirmed: (receipt: TransactionReceipt) => void,
    onError?: (error: Error) => void
  ): () => void {
    if (!this.config.wsUrl) {
      throw new Error('WebSocket URL not configured');
    }

    const ws = new WebSocket(this.config.wsUrl);

    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          jsonrpc: '2.0',
          method: 'eth_subscribe',
          params: ['newHeads'],
          id: 1,
        })
      );
    };

    ws.onmessage = async (event) => {
      const data = JSON.parse(event.data);
      if (data.method === 'eth_subscription') {
        // Check transaction status
        const receipt = await this.getTransactionReceipt(txId);
        if (receipt && (receipt.success || receipt.error)) {
          onConfirmed(receipt);
          ws.close();
        }
      }
    };

    ws.onerror = (_event) => {
      if (onError) {
        onError(new Error('WebSocket error'));
      }
      ws.close();
    };

    return () => ws.close();
  }

  /**
   * Submit a transaction to the blockchain
   */
  private async submitTransaction(method: string, params: any): Promise<string> {
    const response = await fetch(this.config.rpcUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: `dchat_${method}`,
        params: [params],
        id: 1,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to submit transaction: ${response.status}`);
    }

    const responseData = (await response.json()) as any;
    if (responseData.error) {
      throw new Error(`RPC error: ${responseData.error.message}`);
    }

    return responseData.result.tx_id;
  }
}

/**
 * Hash content using SHA-256
 */
export function hashContent(content: string): string {
  return createHash('sha256').update(content).digest('hex');
}
