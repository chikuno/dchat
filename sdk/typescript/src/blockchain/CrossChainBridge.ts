// Cross-chain bridge for TypeScript SDK
// Coordinates atomic operations between chat and currency chains

import axios, { AxiosInstance } from 'axios';
import { ChatChainClient } from './ChatChainClient';
import { CurrencyChainClient } from './CurrencyChainClient';

export enum CrossChainStatus {
  Pending = 'pending',
  ChatChainConfirmed = 'chat_chain_confirmed',
  CurrencyChainConfirmed = 'currency_chain_confirmed',
  AtomicSuccess = 'atomic_success',
  RolledBack = 'rolled_back',
  Failed = 'failed',
}

export interface CrossChainTransaction {
  id: string;
  operation: string;
  user_id: string;
  chat_chain_tx?: string;
  currency_chain_tx?: string;
  status: CrossChainStatus;
  created_at: number;
  finalized_at?: number;
}

export class CrossChainBridge {
  private client: AxiosInstance;

  constructor(
    _chatChain: ChatChainClient,
    _currencyChain: CurrencyChainClient,
    bridgeUrl: string = 'http://localhost:8548',
  ) {
    this.client = axios.create({
      baseURL: bridgeUrl,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  async registerUserWithStake(
    userId: string,
    publicKey: Buffer,
    stakeAmount: number,
  ): Promise<CrossChainTransaction> {
    const response = await this.client.post('/register_user_with_stake', {
      user_id: userId,
      public_key: publicKey.toString('base64'),
      stake_amount: stakeAmount,
    });
    return response.data;
  }

  async createChannelWithFee(owner: string, channelName: string, creationFee: number): Promise<CrossChainTransaction> {
    const response = await this.client.post('/create_channel_with_fee', {
      owner,
      channel_name: channelName,
      creation_fee: creationFee,
    });
    return response.data;
  }

  async getStatus(bridgeTxId: string): Promise<CrossChainTransaction | null> {
    try {
      const response = await this.client.get(`/status/${bridgeTxId}`);
      return response.data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async getUserTransactions(userId: string): Promise<CrossChainTransaction[]> {
    const response = await this.client.get(`/user_transactions/${userId}`);
    return response.data;
  }

  async waitForAtomicCompletion(
    bridgeTxId: string,
    maxWaitMs: number = 60000,
  ): Promise<CrossChainTransaction> {
    const startTime = Date.now();
    while (Date.now() - startTime < maxWaitMs) {
      const tx = await this.getStatus(bridgeTxId);
      if (tx && tx.status === CrossChainStatus.AtomicSuccess) {
        return tx;
      }
      if (tx && (tx.status === CrossChainStatus.Failed || tx.status === CrossChainStatus.RolledBack)) {
        throw new Error(`Cross-chain transaction ${bridgeTxId} failed with status: ${tx.status}`);
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    throw new Error(`Cross-chain transaction ${bridgeTxId} did not complete within ${maxWaitMs}ms`);
  }
}

export default CrossChainBridge;
