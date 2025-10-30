/// Currency Chain client for Dart SDK
/// Handles payments, staking, rewards on currency chain

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'client.dart';

/// Currency Chain transaction types
enum CurrencyChainTxType {
  payment,
  stake,
  unstake,
  reward,
  slash,
  swap,
}

/// Wallet information
class Wallet {
  final String userId;
  final int balance;
  final int staked;
  final int rewardsPending;

  Wallet({
    required this.userId,
    required this.balance,
    required this.staked,
    required this.rewardsPending,
  });

  factory Wallet.fromJson(Map<String, dynamic> json) {
    return Wallet(
      userId: json['user_id'] as String,
      balance: json['balance'] as int,
      staked: json['staked'] as int,
      rewardsPending: json['rewards_pending'] as int,
    );
  }
}

/// Currency Chain client for economics operations
class CurrencyChainClient extends BlockchainClient {
  /// Create new currency chain client
  CurrencyChainClient({
    String rpcUrl = 'http://localhost:8546',
    String? wsUrl,
  }) : super(
    config: BlockchainConfig(
      rpcUrl: rpcUrl,
      wsUrl: wsUrl,
    ),
  );

  /// Create wallet for user
  Future<Wallet> createWallet(String userId, int initialBalance) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/currency/create_wallet'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'user_id': userId,
        'initial_balance': initialBalance,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to create wallet: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return Wallet.fromJson(data['wallet']);
  }

  /// Get wallet balance
  Future<Wallet> getWallet(String userId) async {
    final response = await http.get(
      Uri.parse('${config.rpcUrl}/currency/wallet/$userId'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to get wallet: ${response.body}');
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return Wallet.fromJson(data);
  }

  /// Transfer tokens between users
  Future<Map<String, dynamic>> transfer(
    String from,
    String to,
    int amount,
  ) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/currency/transfer'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'from': from,
        'to': to,
        'amount': amount,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to transfer tokens: ${response.body}');
    }

    return jsonDecode(response.body) as Map<String, dynamic>;
  }

  /// Stake tokens for rewards
  Future<Map<String, dynamic>> stake(
    String userId,
    int amount,
    int lockDurationSeconds,
  ) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/currency/stake'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'user_id': userId,
        'amount': amount,
        'lock_duration_seconds': lockDurationSeconds,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to stake tokens: ${response.body}');
    }

    return jsonDecode(response.body) as Map<String, dynamic>;
  }

  /// Claim rewards
  Future<Map<String, dynamic>> claimRewards(String userId) async {
    final response = await http.post(
      Uri.parse('${config.rpcUrl}/currency/claim_rewards'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'user_id': userId,
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to claim rewards: ${response.body}');
    }

    return jsonDecode(response.body) as Map<String, dynamic>;
  }

  /// Get user transaction history
  Future<List<Map<String, dynamic>>> getUserTransactions(String userId) async {
    final response = await http.get(
      Uri.parse('${config.rpcUrl}/currency/transactions/$userId'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to get transactions: ${response.body}');
    }

    final data = jsonDecode(response.body) as List<dynamic>;
    return data.cast<Map<String, dynamic>>();
  }
}
