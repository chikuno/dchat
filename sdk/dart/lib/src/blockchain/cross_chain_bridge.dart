/// Cross-chain bridge client for Dart SDK
/// Coordinates atomic operations between chat and currency chains

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'chat_chain_client.dart';
import 'currency_chain_client.dart';

/// Cross-chain transaction status
enum CrossChainStatus {
  pending,
  chatChainConfirmed,
  currencyChainConfirmed,
  atomicSuccess,
  rolledBack,
  failed,
}

/// Cross-chain transaction
class CrossChainTransaction {
  final String id;
  final String operation;
  final String userId;
  final String? chatChainTx;
  final String? currencyChainTx;
  final CrossChainStatus status;
  final int createdAt;
  final int? finalizedAt;

  CrossChainTransaction({
    required this.id,
    required this.operation,
    required this.userId,
    this.chatChainTx,
    this.currencyChainTx,
    required this.status,
    required this.createdAt,
    this.finalizedAt,
  });

  factory CrossChainTransaction.fromJson(Map<String, dynamic> json) {
    return CrossChainTransaction(
      id: json['id'] as String,
      operation: json['operation'] as String,
      userId: json['user_id'] as String,
      chatChainTx: json['chat_chain_tx'] as String?,
      currencyChainTx: json['currency_chain_tx'] as String?,
      status: CrossChainStatus.values[json['status'] as int],
      createdAt: json['created_at'] as int,
      finalizedAt: json['finalized_at'] as int?,
    );
  }
}

/// Bridge for coordinating atomic transactions between chains
class CrossChainBridge {
  final ChatChainClient chatChain;
  final CurrencyChainClient currencyChain;
  final String bridgeUrl;

  CrossChainBridge({
    required this.chatChain,
    required this.currencyChain,
    this.bridgeUrl = 'http://localhost:8548',
  });

  /// Register user with initial stake (atomic operation)
  Future<CrossChainTransaction> registerUserWithStake(
    String userId,
    List<int> publicKey,
    int stakeAmount,
  ) async {
    final response = await http.post(
      Uri.parse('$bridgeUrl/register_user_with_stake'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'user_id': userId,
        'public_key': base64Encode(publicKey),
        'stake_amount': stakeAmount,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to register user with stake: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return CrossChainTransaction.fromJson(data);
  }

  /// Create channel with fee (atomic operation)
  Future<CrossChainTransaction> createChannelWithFee(
    String owner,
    String channelName,
    int creationFee,
  ) async {
    final response = await http.post(
      Uri.parse('$bridgeUrl/create_channel_with_fee'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'owner': owner,
        'channel_name': channelName,
        'creation_fee': creationFee,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to create channel with fee: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return CrossChainTransaction.fromJson(data);
  }

  /// Get cross-chain transaction status
  Future<CrossChainTransaction?> getStatus(String bridgeTxId) async {
    final response = await http.get(
      Uri.parse('$bridgeUrl/status/$bridgeTxId'),
    );

    if (response.statusCode == 404) {
      return null;
    }

    if (response.statusCode != 200) {
      throw Exception('Failed to get status: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return CrossChainTransaction.fromJson(data);
  }

  /// Get all cross-chain transactions for user
  Future<List<CrossChainTransaction>> getUserTransactions(String userId) async {
    final response = await http.get(
      Uri.parse('$bridgeUrl/user_transactions/$userId'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to get transactions: ${response.body}');
    }

    final data = jsonDecode(response.body) as List<dynamic>;
    return data
        .map((item) => CrossChainTransaction.fromJson(item as Map<String, dynamic>))
        .toList();
  }
}
