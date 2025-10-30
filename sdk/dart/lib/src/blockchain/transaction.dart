/// Transaction types and models for blockchain operations
library;



/// Transaction status enum
enum TransactionStatus {
  pending,
  confirmed,
  failed,
  timedOut,
}

/// Transaction receipt containing confirmation details
class TransactionReceipt {
  final String txId;
  final String txHash;
  final bool success;
  final int? blockHeight;
  final String? blockHash;
  final DateTime timestamp;
  final String? error;

  TransactionReceipt({
    required this.txId,
    required this.txHash,
    required this.success,
    this.blockHeight,
    this.blockHash,
    required this.timestamp,
    this.error,
  });

  factory TransactionReceipt.fromJson(Map<String, dynamic> json) {
    return TransactionReceipt(
      txId: json['tx_id'] as String,
      txHash: json['tx_hash'] as String,
      success: json['success'] as bool,
      blockHeight: json['block_height'] as int?,
      blockHash: json['block_hash'] as String?,
      timestamp: DateTime.parse(json['timestamp'] as String),
      error: json['error'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'tx_id': txId,
      'tx_hash': txHash,
      'success': success,
      if (blockHeight != null) 'block_height': blockHeight,
      if (blockHash != null) 'block_hash': blockHash,
      'timestamp': timestamp.toIso8601String(),
      if (error != null) 'error': error,
    };
  }
}

/// Register user transaction
class RegisterUserTx {
  final String userId;
  final String username;
  final String publicKey;
  final DateTime timestamp;
  final int initialReputation;

  RegisterUserTx({
    required this.userId,
    required this.username,
    required this.publicKey,
    required this.timestamp,
    this.initialReputation = 100,
  });

  Map<String, dynamic> toJson() {
    return {
      'user_id': userId,
      'username': username,
      'public_key': publicKey,
      'timestamp': timestamp.toIso8601String(),
      'initial_reputation': initialReputation,
    };
  }
}

/// Send direct message transaction
class SendDirectMessageTx {
  final String messageId;
  final String senderId;
  final String recipientId;
  final String contentHash;
  final int payloadSize;
  final DateTime timestamp;
  final String? relayNodeId;

  SendDirectMessageTx({
    required this.messageId,
    required this.senderId,
    required this.recipientId,
    required this.contentHash,
    required this.payloadSize,
    required this.timestamp,
    this.relayNodeId,
  });

  Map<String, dynamic> toJson() {
    return {
      'message_id': messageId,
      'sender_id': senderId,
      'recipient_id': recipientId,
      'content_hash': contentHash,
      'payload_size': payloadSize,
      'timestamp': timestamp.toIso8601String(),
      if (relayNodeId != null) 'relay_node_id': relayNodeId,
    };
  }
}

/// Channel visibility enum
enum ChannelVisibility {
  public,
  private,
  tokenGated,
}

/// Create channel transaction
class CreateChannelTx {
  final String channelId;
  final String name;
  final String description;
  final String creatorId;
  final ChannelVisibility visibility;
  final DateTime timestamp;
  final String? tokenRequirement;

  CreateChannelTx({
    required this.channelId,
    required this.name,
    required this.description,
    required this.creatorId,
    this.visibility = ChannelVisibility.public,
    required this.timestamp,
    this.tokenRequirement,
  });

  Map<String, dynamic> toJson() {
    return {
      'channel_id': channelId,
      'name': name,
      'description': description,
      'creator_id': creatorId,
      'visibility': visibility.name,
      'timestamp': timestamp.toIso8601String(),
      if (tokenRequirement != null) 'token_requirement': tokenRequirement,
    };
  }
}

/// Post to channel transaction
class PostToChannelTx {
  final String messageId;
  final String channelId;
  final String senderId;
  final String contentHash;
  final int payloadSize;
  final DateTime timestamp;

  PostToChannelTx({
    required this.messageId,
    required this.channelId,
    required this.senderId,
    required this.contentHash,
    required this.payloadSize,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() {
    return {
      'message_id': messageId,
      'channel_id': channelId,
      'sender_id': senderId,
      'content_hash': contentHash,
      'payload_size': payloadSize,
      'timestamp': timestamp.toIso8601String(),
    };
  }
}
