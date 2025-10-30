/// Blockchain client for submitting transactions and querying chain state
library;

import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:web_socket_channel/web_socket_channel.dart';
import 'transaction.dart';

/// Configuration for blockchain client
class BlockchainConfig {
  final String rpcUrl;
  final String? wsUrl;
  final int confirmationBlocks;
  final Duration confirmationTimeout;
  final int maxRetries;

  const BlockchainConfig({
    required this.rpcUrl,
    this.wsUrl,
    this.confirmationBlocks = 6,
    this.confirmationTimeout = const Duration(seconds: 300),
    this.maxRetries = 3,
  });

  factory BlockchainConfig.local() {
    return const BlockchainConfig(
      rpcUrl: 'http://localhost:8545',
      wsUrl: 'ws://localhost:8546',
    );
  }
}

/// Blockchain client for transaction submission and confirmation
class BlockchainClient {
  final BlockchainConfig config;
  final http.Client _httpClient;
  WebSocketChannel? _wsChannel;
  final Map<String, TransactionReceipt> _transactionCache = {};

  BlockchainClient({
    required this.config,
    http.Client? httpClient,
  }) : _httpClient = httpClient ?? http.Client();

  /// Register a new user on-chain
  Future<String> registerUser({
    required String userId,
    required String username,
    required String publicKey,
  }) async {
    final tx = RegisterUserTx(
      userId: userId,
      username: username,
      publicKey: publicKey,
      timestamp: DateTime.now().toUtc(),
    );

    return await _submitTransaction('register_user', tx.toJson());
  }

  /// Send a direct message on-chain
  Future<String> sendDirectMessage({
    required String messageId,
    required String senderId,
    required String recipientId,
    required String contentHash,
    required int payloadSize,
    String? relayNodeId,
  }) async {
    final tx = SendDirectMessageTx(
      messageId: messageId,
      senderId: senderId,
      recipientId: recipientId,
      contentHash: contentHash,
      payloadSize: payloadSize,
      timestamp: DateTime.now().toUtc(),
      relayNodeId: relayNodeId,
    );

    return await _submitTransaction('send_direct_message', tx.toJson());
  }

  /// Create a new channel on-chain
  Future<String> createChannel({
    required String channelId,
    required String name,
    required String description,
    required String creatorId,
    ChannelVisibility visibility = ChannelVisibility.public,
    String? tokenRequirement,
  }) async {
    final tx = CreateChannelTx(
      channelId: channelId,
      name: name,
      description: description,
      creatorId: creatorId,
      visibility: visibility,
      timestamp: DateTime.now().toUtc(),
      tokenRequirement: tokenRequirement,
    );

    return await _submitTransaction('create_channel', tx.toJson());
  }

  /// Post a message to a channel on-chain
  Future<String> postToChannel({
    required String messageId,
    required String channelId,
    required String senderId,
    required String contentHash,
    required int payloadSize,
  }) async {
    final tx = PostToChannelTx(
      messageId: messageId,
      channelId: channelId,
      senderId: senderId,
      contentHash: contentHash,
      payloadSize: payloadSize,
      timestamp: DateTime.now().toUtc(),
    );

    return await _submitTransaction('post_to_channel', tx.toJson());
  }

  /// Wait for transaction confirmation
  Future<TransactionReceipt> waitForConfirmation(String txId) async {
    // Check cache first
    if (_transactionCache.containsKey(txId)) {
      final cached = _transactionCache[txId]!;
      if (cached.success || cached.error != null) {
        return cached;
      }
    }

    final startTime = DateTime.now();
    final deadline = startTime.add(config.confirmationTimeout);

    while (DateTime.now().isBefore(deadline)) {
      try {
        final receipt = await getTransactionReceipt(txId);
        if (receipt != null) {
          _transactionCache[txId] = receipt;
          
          if (receipt.success) {
            return receipt;
          } else if (receipt.error != null) {
            throw Exception('Transaction failed: ${receipt.error}');
          }
        }
      } catch (e) {
        // Continue polling on error
      }

      // Poll every 2 seconds
      await Future.delayed(const Duration(seconds: 2));
    }

    throw TimeoutException('Transaction confirmation timed out', config.confirmationTimeout);
  }

  /// Check if a transaction is confirmed
  Future<bool> isTransactionConfirmed(String txId) async {
    try {
      final receipt = await getTransactionReceipt(txId);
      return receipt?.success ?? false;
    } catch (e) {
      return false;
    }
  }

  /// Get transaction receipt
  Future<TransactionReceipt?> getTransactionReceipt(String txId) async {
    try {
      final response = await _httpClient.post(
        Uri.parse(config.rpcUrl),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode({
          'jsonrpc': '2.0',
          'method': 'eth_getTransactionReceipt',
          'params': [txId],
          'id': 1,
        }),
      );

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        if (data['result'] != null) {
          return TransactionReceipt.fromJson(data['result']);
        }
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// Get current block number
  Future<int> getBlockNumber() async {
    final response = await _httpClient.post(
      Uri.parse(config.rpcUrl),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'jsonrpc': '2.0',
        'method': 'eth_blockNumber',
        'params': [],
        'id': 1,
      }),
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return int.parse(data['result'] as String, radix: 16);
    }

    throw Exception('Failed to get block number: ${response.statusCode}');
  }

  /// Subscribe to transaction confirmations via WebSocket
  Stream<TransactionReceipt> subscribeToConfirmations(String txId) {
    if (config.wsUrl == null) {
      throw Exception('WebSocket URL not configured');
    }

    final controller = StreamController<TransactionReceipt>();
    
    _wsChannel = WebSocketChannel.connect(Uri.parse(config.wsUrl!));
    
    // Subscribe to transaction updates
    _wsChannel!.sink.add(jsonEncode({
      'jsonrpc': '2.0',
      'method': 'eth_subscribe',
      'params': ['newHeads'],
      'id': 1,
    }));

    _wsChannel!.stream.listen(
      (message) {
        final data = jsonDecode(message);
        if (data['method'] == 'eth_subscription') {
          // Check transaction status
          getTransactionReceipt(txId).then((receipt) {
            if (receipt != null && (receipt.success || receipt.error != null)) {
              controller.add(receipt);
              controller.close();
              _wsChannel?.sink.close();
            }
          });
        }
      },
      onError: (error) {
        controller.addError(error);
        controller.close();
      },
      onDone: () {
        controller.close();
      },
    );

    return controller.stream;
  }

  /// Submit a transaction to the blockchain
  Future<String> _submitTransaction(String method, Map<String, dynamic> params) async {
    final response = await _httpClient.post(
      Uri.parse(config.rpcUrl),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'jsonrpc': '2.0',
        'method': 'dchat_$method',
        'params': [params],
        'id': 1,
      }),
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      if (data['error'] != null) {
        throw Exception('RPC error: ${data['error']['message']}');
      }
      return data['result']['tx_id'] as String;
    }

    throw Exception('Failed to submit transaction: ${response.statusCode}');
  }

  /// Close the client and cleanup resources
  void dispose() {
    _httpClient.close();
    _wsChannel?.sink.close();
  }
}
