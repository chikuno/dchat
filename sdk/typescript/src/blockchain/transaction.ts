/**
 * Blockchain transaction types for dchat
 */

export enum TransactionStatus {
  Pending = 'Pending',
  Confirmed = 'Confirmed',
  Failed = 'Failed',
  TimedOut = 'TimedOut',
}

export interface TransactionReceipt {
  txId: string;
  txHash: string;
  success: boolean;
  blockHeight?: number;
  blockHash?: string;
  timestamp: Date;
  error?: string;
}

export enum ChannelVisibility {
  Public = 'Public',
  Private = 'Private',
  TokenGated = 'TokenGated',
}

export interface RegisterUserTx {
  userId: string;
  username: string;
  publicKey: string;
  timestamp: Date;
  initialReputation: number;
}

export interface SendDirectMessageTx {
  messageId: string;
  senderId: string;
  recipientId: string;
  contentHash: string;
  payloadSize: number;
  timestamp: Date;
  relayNodeId?: string;
}

export interface CreateChannelTx {
  channelId: string;
  name: string;
  description: string;
  creatorId: string;
  visibility: ChannelVisibility;
  timestamp: Date;
  tokenRequirement?: string;
}

export interface PostToChannelTx {
  messageId: string;
  channelId: string;
  senderId: string;
  contentHash: string;
  payloadSize: number;
  timestamp: Date;
}

export interface Transaction {
  txId: string;
  txType: string;
  payload: Uint8Array;
  txHash: string;
  status: TransactionStatus;
  submittedAt: Date;
  confirmedAt?: Date;
  fee?: number;
}
