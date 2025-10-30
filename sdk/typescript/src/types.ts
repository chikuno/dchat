/**
 * Core types for dchat SDK
 */

export interface Identity {
  userId: string;
  username: string;
  publicKey: string;
  displayName?: string;
  bio?: string;
  reputation: number;
  createdAt: Date;
  verified: boolean;
  badges: string[];
}

export enum MessageStatus {
  Created = 'Created',
  Sent = 'Sent',
  Delivered = 'Delivered',
  Read = 'Read',
  Failed = 'Failed',
  Expired = 'Expired',
}

export type MessageContent =
  | { type: 'Text'; text: string }
  | { type: 'Image'; data: Uint8Array; mimeType: string }
  | { type: 'File'; data: Uint8Array; filename: string; mimeType: string }
  | { type: 'Audio'; data: Uint8Array; durationMs: number }
  | { type: 'Video'; data: Uint8Array; durationMs: number; width: number; height: number }
  | { type: 'Sticker'; packId: string; stickerId: string }
  | { type: 'System'; text: string };

export interface Message {
  id: string;
  senderId: string;
  recipientId?: string;
  channelId?: string;
  content: MessageContent;
  encryptedPayload: Uint8Array;
  timestamp: Date;
  sequence?: number;
  status: MessageStatus;
  expiresAt?: Date;
  size: number;
}

export interface RelayStats {
  connectedPeers: number;
  messagesRelayed: number;
  uptimePercent: number;
  reputationScore: number;
}
