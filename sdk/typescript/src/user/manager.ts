/**
 * User management for creating users and handling profiles
 */

import { v4 as uuidv4 } from 'uuid';
import { BlockchainClient, hashContent } from '../blockchain/client';
import { generateKeyPair } from '../crypto/keypair';
import {
  CreateUserResponse,
  DirectMessageResponse,
  CreateChannelResponse,
} from './models';

export class UserManager {
  constructor(
    private blockchain: BlockchainClient
  ) {}

  /**
   * Create a new user with blockchain registration
   */
  async createUser(username: string): Promise<CreateUserResponse> {
    // Generate unique user ID
    const userId = uuidv4();

    // Generate Ed25519 key pair
    const keyPair = generateKeyPair();

    // Submit blockchain transaction
    const txId = await this.blockchain.registerUser(
      userId,
      username,
      keyPair.publicKey
    );

    // Wait for blockchain confirmation
    const receipt = await this.blockchain.waitForConfirmation(txId);
    const onChainConfirmed = receipt.success;

    // Return response with actual blockchain status
    return {
      userId,
      username,
      publicKey: keyPair.publicKey,
      privateKey: keyPair.privateKey,
      createdAt: new Date().toISOString(),
      onChainConfirmed,
      txId,
    };
  }

  /**
   * Send a direct message
   */
  async sendDirectMessage(
    senderId: string,
    recipientId: string,
    content: string,
    relayNodeId?: string
  ): Promise<DirectMessageResponse> {
    // Generate message ID
    const messageId = uuidv4();

    // Hash the content
    const contentHash = hashContent(content);

    // Submit blockchain transaction
    const txId = await this.blockchain.sendDirectMessage(
      messageId,
      senderId,
      recipientId,
      contentHash,
      content.length,
      relayNodeId
    );

    // Wait for confirmation
    const receipt = await this.blockchain.waitForConfirmation(txId);
    const onChainConfirmed = receipt.success;

    return {
      messageId,
      senderId,
      recipientId,
      contentHash,
      createdAt: new Date().toISOString(),
      onChainConfirmed,
      txId,
    };
  }

  /**
   * Create a new channel
   */
  async createChannel(
    creatorId: string,
    channelName: string,
    description?: string
  ): Promise<CreateChannelResponse> {
    // Generate channel ID
    const channelId = uuidv4();

    // Submit blockchain transaction
    const txId = await this.blockchain.createChannel(
      channelId,
      channelName,
      description || '',
      creatorId
    );

    // Wait for confirmation
    const receipt = await this.blockchain.waitForConfirmation(txId);
    const onChainConfirmed = receipt.success;

    return {
      channelId,
      name: channelName,
      description,
      creatorId,
      createdAt: new Date().toISOString(),
      onChainConfirmed,
      txId,
    };
  }

  /**
   * Post a message to a channel
   */
  async postToChannel(
    senderId: string,
    channelId: string,
    content: string
  ): Promise<DirectMessageResponse> {
    // Generate message ID
    const messageId = uuidv4();

    // Hash the content
    const contentHash = hashContent(content);

    // Submit blockchain transaction
    const txId = await this.blockchain.postToChannel(
      messageId,
      channelId,
      senderId,
      contentHash,
      content.length
    );

    // Wait for confirmation
    const receipt = await this.blockchain.waitForConfirmation(txId);
    const onChainConfirmed = receipt.success;

    return {
      messageId,
      senderId,
      recipientId: channelId, // Using channelId as recipient
      contentHash,
      createdAt: new Date().toISOString(),
      onChainConfirmed,
      txId,
    };
  }
}
