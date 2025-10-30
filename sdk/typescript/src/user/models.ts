/**
 * User management models and responses
 */

export interface CreateUserResponse {
  userId: string;
  username: string;
  publicKey: string;
  privateKey: string;
  createdAt: string;
  onChainConfirmed: boolean;
  txId?: string;
}

export interface UserProfile {
  userId: string;
  username: string;
  publicKey: string;
  createdAt: string;
  reputation: number;
  onChainConfirmed: boolean;
}

export interface DirectMessageResponse {
  messageId: string;
  senderId: string;
  recipientId: string;
  contentHash: string;
  createdAt: string;
  onChainConfirmed: boolean;
  txId?: string;
}

export interface CreateChannelResponse {
  channelId: string;
  name: string;
  description?: string;
  creatorId: string;
  createdAt: string;
  onChainConfirmed: boolean;
  txId?: string;
}

export interface ChannelMessage {
  messageId: string;
  channelId: string;
  senderId: string;
  content: string;
  contentHash: string;
  createdAt: string;
  onChainConfirmed: boolean;
}

export interface DirectMessage {
  messageId: string;
  senderId: string;
  recipientId: string;
  content: string;
  contentHash: string;
  createdAt: string;
  onChainConfirmed: boolean;
}
