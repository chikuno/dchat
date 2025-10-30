// Chat Chain client for TypeScript SDK
// Handles identity, messaging, channels, governance on chat chain

import axios, { AxiosInstance } from 'axios';

export enum ChatChainTxType {
  RegisterUser = 'register_user',
  SendDirectMessage = 'send_direct_message',
  CreateChannel = 'create_channel',
  PostToChannel = 'post_to_channel',
  UpdateReputation = 'update_reputation',
  Governance = 'governance',
}

export interface ChatChainTransaction {
  id: string;
  tx_type: ChatChainTxType;
  sender: string;
  data: Record<string, any>;
  status: 'pending' | 'confirmed' | 'failed';
  confirmations: number;
  block_height: number;
  created_at: number;
}

export class ChatChainClient {
  private client: AxiosInstance;

  constructor(rpcUrl: string = 'http://localhost:8545') {
    this.client = axios.create({
      baseURL: rpcUrl,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  async registerUser(userId: string, publicKey: Buffer): Promise<ChatChainTransaction> {
    const response = await this.client.post('/chat/register_user', {
      user_id: userId,
      public_key: publicKey.toString('base64'),
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async sendDirectMessage(
    sender: string,
    recipient: string,
    messageId: string,
  ): Promise<ChatChainTransaction> {
    const response = await this.client.post('/chat/send_message', {
      sender,
      recipient,
      message_id: messageId,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async createChannel(owner: string, channelId: string, name: string): Promise<ChatChainTransaction> {
    const response = await this.client.post('/chat/create_channel', {
      owner,
      channel_id: channelId,
      name,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async postToChannel(sender: string, channelId: string, messageId: string): Promise<ChatChainTransaction> {
    const response = await this.client.post('/chat/post_message', {
      sender,
      channel_id: channelId,
      message_id: messageId,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async getReputation(userId: string): Promise<number> {
    const response = await this.client.get(`/chat/reputation/${userId}`);
    return response.data.reputation;
  }

  async getUserTransactions(userId: string): Promise<ChatChainTransaction[]> {
    const response = await this.client.get(`/chat/transactions/${userId}`);
    return response.data;
  }

  async getTransaction(txId: string): Promise<ChatChainTransaction | null> {
    try {
      const response = await this.client.get(`/chat/transaction/${txId}`);
      return response.data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async waitForConfirmation(txId: string, confirmations: number = 6, maxWaitMs: number = 30000): Promise<ChatChainTransaction> {
    const startTime = Date.now();
    while (Date.now() - startTime < maxWaitMs) {
      const tx = await this.getTransaction(txId);
      if (tx && tx.confirmations >= confirmations) {
        return tx;
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    throw new Error(`Transaction ${txId} did not confirm within ${maxWaitMs}ms`);
  }
}

export default ChatChainClient;
