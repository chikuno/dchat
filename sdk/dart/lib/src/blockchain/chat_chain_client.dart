/// Chat Chain client for Dart SDK
/// Handles identity, messaging, channels, governance on chat chain

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'client.dart';
import 'transaction.dart';

/// Chat Chain transaction types specific to messaging and identity
enum ChatChainTxType {
  registerUser,
  sendDirectMessage,
  createChannel,
  postToChannel,
  updateReputation,
  governance,
}

/// Chat Chain client for identity and messaging operations
class ChatChainClient extends BlockchainClient {
  /// Create new chat chain client
  ChatChainClient({
    String rpcUrl = 'http://localhost:8545',
    String? wsUrl,
  }) : super(
    config: BlockchainConfig(
      rpcUrl: rpcUrl,
      wsUrl: wsUrl,
    ),
  );

  /// Register user identity on chat chain
  @override
  Future<String> registerUser({
    required String userId,
    required String username,
    required String publicKey,
  }) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/chat/register_user'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'user_id': userId,
        'username': username,
        'public_key': publicKey,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to register user: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return data['tx_id'] as String;
  }

  /// Send direct message transaction
  @override
  Future<String> sendDirectMessage({
    required String messageId,
    required String senderId,
    required String recipientId,
    required String contentHash,
    required int payloadSize,
    String? relayNodeId,
  }) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/chat/send_message'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'message_id': messageId,
        'sender_id': senderId,
        'recipient_id': recipientId,
        'content_hash': contentHash,
        'payload_size': payloadSize,
        'relay_node_id': relayNodeId,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to send message: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return data['tx_id'] as String;
  }

  /// Create channel on chat chain
  @override
  Future<String> createChannel({
    required String channelId,
    required String creatorId,
    required String name,
    required String description,
    String? tokenRequirement,
    ChannelVisibility visibility = ChannelVisibility.public,
  }) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/chat/create_channel'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'channel_id': channelId,
        'creator_id': creatorId,
        'name': name,
        'description': description,
        'token_requirement': tokenRequirement,
        'visibility': visibility.toString().split('.').last,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to create channel: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return data['tx_id'] as String;
  }

  /// Post message to channel on chat chain
  @override
  Future<String> postToChannel({
    required String channelId,
    required String contentHash,
    required String messageId,
    required int payloadSize,
    required String senderId,
  }) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/chat/post_message'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'channel_id': channelId,
        'sender_id': senderId,
        'message_id': messageId,
        'content_hash': contentHash,
        'payload_size': payloadSize,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to post message: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return data['tx_id'] as String;
  }

  /// Get user reputation
  Future<int> getReputation(String userId) async {
    final response = await http.get(
      Uri.parse('${config.rpcUrl}/chat/reputation/$userId'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to get reputation: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return data['reputation'] as int;
  }

  /// Get user transaction history
  Future<List<Map<String, dynamic>>> getUserTransactions(String userId) async {
    final response = await http.get(
      Uri.parse('${config.rpcUrl}/chat/transactions/$userId'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to get transactions: ${response.body}');
    }

    final data = jsonDecode(response.body) as List<dynamic>;
    return data.cast<Map<String, dynamic>>();
  }
}
