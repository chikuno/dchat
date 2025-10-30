/// User management for creating users and handling profiles
library;

import 'package:uuid/uuid.dart';
import '../blockchain/client.dart';
import '../crypto/keypair.dart';
import 'models.dart';

/// User manager for user operations
class UserManager {
  final BlockchainClient blockchain;
  final String baseUrl;
  final Uuid _uuid = const Uuid();

  UserManager({
    required this.blockchain,
    required this.baseUrl,
  });

  /// Create a new user with blockchain registration
  Future<CreateUserResponse> createUser(String username) async {
    // Generate unique user ID
    final userId = _uuid.v4();

    // Generate Ed25519 key pair
    final keyPair = KeyPair.generate();

    // Submit blockchain transaction
    final txId = await blockchain.registerUser(
      userId: userId,
      username: username,
      publicKey: keyPair.publicKeyHex,
    );

    // Wait for blockchain confirmation
    final receipt = await blockchain.waitForConfirmation(txId);
    final onChainConfirmed = receipt.success;

    // Return response with actual blockchain status
    return CreateUserResponse(
      userId: userId,
      username: username,
      publicKey: keyPair.publicKeyHex,
      privateKey: keyPair.privateKeyHex,
      createdAt: DateTime.now().toUtc().toIso8601String(),
      onChainConfirmed: onChainConfirmed,
      txId: txId,
    );
  }

  /// Get user profile by user ID
  Future<UserProfile?> getUserProfile(String userId) async {
    // Implementation would query the backend/database
    // For now, this is a placeholder
    throw UnimplementedError('getUserProfile not yet implemented');
  }

  /// Send a direct message
  Future<DirectMessageResponse> sendDirectMessage({
    required String senderId,
    required String recipientId,
    required String content,
    String? relayNodeId,
  }) async {
    // Generate message ID
    final messageId = _uuid.v4();

    // Hash the content
    final contentHash = hashContent(content);

    // Submit blockchain transaction
    final txId = await blockchain.sendDirectMessage(
      messageId: messageId,
      senderId: senderId,
      recipientId: recipientId,
      contentHash: contentHash,
      payloadSize: content.length,
      relayNodeId: relayNodeId,
    );

    // Wait for confirmation
    final receipt = await blockchain.waitForConfirmation(txId);
    final onChainConfirmed = receipt.success;

    return DirectMessageResponse(
      messageId: messageId,
      senderId: senderId,
      recipientId: recipientId,
      contentHash: contentHash,
      createdAt: DateTime.now().toUtc().toIso8601String(),
      onChainConfirmed: onChainConfirmed,
      txId: txId,
    );
  }

  /// Create a new channel
  Future<CreateChannelResponse> createChannel({
    required String creatorId,
    required String channelName,
    String? description,
  }) async {
    // Generate channel ID
    final channelId = _uuid.v4();

    // Submit blockchain transaction
    final txId = await blockchain.createChannel(
      channelId: channelId,
      name: channelName,
      description: description ?? '',
      creatorId: creatorId,
    );

    // Wait for confirmation
    final receipt = await blockchain.waitForConfirmation(txId);
    final onChainConfirmed = receipt.success;

    return CreateChannelResponse(
      channelId: channelId,
      name: channelName,
      description: description,
      creatorId: creatorId,
      createdAt: DateTime.now().toUtc().toIso8601String(),
      onChainConfirmed: onChainConfirmed,
      txId: txId,
    );
  }

  /// Post a message to a channel
  Future<DirectMessageResponse> postToChannel({
    required String senderId,
    required String channelId,
    required String content,
  }) async {
    // Generate message ID
    final messageId = _uuid.v4();

    // Hash the content
    final contentHash = hashContent(content);

    // Submit blockchain transaction
    final txId = await blockchain.postToChannel(
      messageId: messageId,
      channelId: channelId,
      senderId: senderId,
      contentHash: contentHash,
      payloadSize: content.length,
    );

    // Wait for confirmation
    final receipt = await blockchain.waitForConfirmation(txId);
    final onChainConfirmed = receipt.success;

    return DirectMessageResponse(
      messageId: messageId,
      senderId: senderId,
      recipientId: channelId, // Using channelId as recipient for response
      contentHash: contentHash,
      createdAt: DateTime.now().toUtc().toIso8601String(),
      onChainConfirmed: onChainConfirmed,
      txId: txId,
    );
  }

  /// Get direct messages for a user
  Future<List<DirectMessage>> getDirectMessages({
    required String userId,
    int limit = 50,
  }) async {
    // Implementation would query the backend/database
    // For now, this is a placeholder
    throw UnimplementedError('getDirectMessages not yet implemented');
  }

  /// Get channel messages
  Future<List<ChannelMessage>> getChannelMessages({
    required String channelId,
    int limit = 50,
  }) async {
    // Implementation would query the backend/database
    // For now, this is a placeholder
    throw UnimplementedError('getChannelMessages not yet implemented');
  }
}
