// Currency Chain client for TypeScript SDK
// Handles payments, staking, rewards on currency chain

import axios, { AxiosInstance } from 'axios';

export enum CurrencyChainTxType {
  Payment = 'payment',
  Stake = 'stake',
  Unstake = 'unstake',
  Reward = 'reward',
  Slash = 'slash',
  Swap = 'swap',
}

export interface Wallet {
  user_id: string;
  balance: number;
  staked: number;
  rewards_pending: number;
}

export interface CurrencyChainTransaction {
  id: string;
  tx_type: CurrencyChainTxType;
  from: string;
  to?: string;
  amount: number;
  status: 'pending' | 'confirmed' | 'failed';
  confirmations: number;
  block_height: number;
  created_at: number;
}

export class CurrencyChainClient {
  private client: AxiosInstance;

  constructor(rpcUrl: string = 'http://localhost:8546') {
    this.client = axios.create({
      baseURL: rpcUrl,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  async createWallet(userId: string, initialBalance: number): Promise<Wallet> {
    const response = await this.client.post('/currency/create_wallet', {
      user_id: userId,
      initial_balance: initialBalance,
    });
    return response.data.wallet;
  }

  async getWallet(userId: string): Promise<Wallet> {
    const response = await this.client.get(`/currency/wallet/${userId}`);
    return response.data;
  }

  async getBalance(userId: string): Promise<number> {
    const wallet = await this.getWallet(userId);
    return wallet.balance;
  }

  async transfer(from: string, to: string, amount: number): Promise<CurrencyChainTransaction> {
    const response = await this.client.post('/currency/transfer', {
      from,
      to,
      amount,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async stake(userId: string, amount: number, lockDurationSeconds: number): Promise<CurrencyChainTransaction> {
    const response = await this.client.post('/currency/stake', {
      user_id: userId,
      amount,
      lock_duration_seconds: lockDurationSeconds,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async claimRewards(userId: string): Promise<CurrencyChainTransaction> {
    const response = await this.client.post('/currency/claim_rewards', {
      user_id: userId,
      timestamp: Math.floor(Date.now() / 1000),
    });
    return response.data;
  }

  async getUserTransactions(userId: string): Promise<CurrencyChainTransaction[]> {
    const response = await this.client.get(`/currency/transactions/${userId}`);
    return response.data;
  }

  async getTransaction(txId: string): Promise<CurrencyChainTransaction | null> {
    try {
      const response = await this.client.get(`/currency/transaction/${txId}`);
      return response.data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async waitForConfirmation(
    txId: string,
    confirmations: number = 6,
    maxWaitMs: number = 30000,
  ): Promise<CurrencyChainTransaction> {
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

export default CurrencyChainClient;
