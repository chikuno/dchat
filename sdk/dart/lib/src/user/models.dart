/// User management models and responses
library;

/// Response for user creation
class CreateUserResponse {
  final String userId;
  final String username;
  final String publicKey;
  final String privateKey;
  final String createdAt;
  final bool onChainConfirmed;
  final String? txId;

  CreateUserResponse({
    required this.userId,
    required this.username,
    required this.publicKey,
    required this.privateKey,
    required this.createdAt,
    required this.onChainConfirmed,
    this.txId,
  });

  factory CreateUserResponse.fromJson(Map<String, dynamic> json) {
    return CreateUserResponse(
      userId: json['user_id'] as String,
      username: json['username'] as String,
      publicKey: json['public_key'] as String,
      privateKey: json['private_key'] as String,
      createdAt: json['created_at'] as String,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
      txId: json['tx_id'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'user_id': userId,
      'username': username,
      'public_key': publicKey,
      'private_key': privateKey,
      'created_at': createdAt,
      'on_chain_confirmed': onChainConfirmed,
      if (txId != null) 'tx_id': txId,
    };
  }
}

/// User profile information
class UserProfile {
  final String userId;
  final String username;
  final String publicKey;
  final String createdAt;
  final int reputation;
  final bool onChainConfirmed;

  UserProfile({
    required this.userId,
    required this.username,
    required this.publicKey,
    required this.createdAt,
    required this.reputation,
    required this.onChainConfirmed,
  });

  factory UserProfile.fromJson(Map<String, dynamic> json) {
    return UserProfile(
      userId: json['user_id'] as String,
      username: json['username'] as String,
      publicKey: json['public_key'] as String,
      createdAt: json['created_at'] as String,
      reputation: json['reputation'] as int,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'user_id': userId,
      'username': username,
      'public_key': publicKey,
      'created_at': createdAt,
      'reputation': reputation,
      'on_chain_confirmed': onChainConfirmed,
    };
  }
}

/// Direct message response
class DirectMessageResponse {
  final String messageId;
  final String senderId;
  final String recipientId;
  final String contentHash;
  final String createdAt;
  final bool onChainConfirmed;
  final String? txId;

  DirectMessageResponse({
    required this.messageId,
    required this.senderId,
    required this.recipientId,
    required this.contentHash,
    required this.createdAt,
    required this.onChainConfirmed,
    this.txId,
  });

  factory DirectMessageResponse.fromJson(Map<String, dynamic> json) {
    return DirectMessageResponse(
      messageId: json['message_id'] as String,
      senderId: json['sender_id'] as String,
      recipientId: json['recipient_id'] as String,
      contentHash: json['content_hash'] as String,
      createdAt: json['created_at'] as String,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
      txId: json['tx_id'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'message_id': messageId,
      'sender_id': senderId,
      'recipient_id': recipientId,
      'content_hash': contentHash,
      'created_at': createdAt,
      'on_chain_confirmed': onChainConfirmed,
      if (txId != null) 'tx_id': txId,
    };
  }
}

/// Channel creation response
class CreateChannelResponse {
  final String channelId;
  final String name;
  final String? description;
  final String creatorId;
  final String createdAt;
  final bool onChainConfirmed;
  final String? txId;

  CreateChannelResponse({
    required this.channelId,
    required this.name,
    this.description,
    required this.creatorId,
    required this.createdAt,
    required this.onChainConfirmed,
    this.txId,
  });

  factory CreateChannelResponse.fromJson(Map<String, dynamic> json) {
    return CreateChannelResponse(
      channelId: json['channel_id'] as String,
      name: json['name'] as String,
      description: json['description'] as String?,
      creatorId: json['creator_id'] as String,
      createdAt: json['created_at'] as String,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
      txId: json['tx_id'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'channel_id': channelId,
      'name': name,
      if (description != null) 'description': description,
      'creator_id': creatorId,
      'created_at': createdAt,
      'on_chain_confirmed': onChainConfirmed,
      if (txId != null) 'tx_id': txId,
    };
  }
}

/// Channel message
class ChannelMessage {
  final String messageId;
  final String channelId;
  final String senderId;
  final String content;
  final String contentHash;
  final String createdAt;
  final bool onChainConfirmed;

  ChannelMessage({
    required this.messageId,
    required this.channelId,
    required this.senderId,
    required this.content,
    required this.contentHash,
    required this.createdAt,
    required this.onChainConfirmed,
  });

  factory ChannelMessage.fromJson(Map<String, dynamic> json) {
    return ChannelMessage(
      messageId: json['message_id'] as String,
      channelId: json['channel_id'] as String,
      senderId: json['sender_id'] as String,
      content: json['content'] as String,
      contentHash: json['content_hash'] as String,
      createdAt: json['created_at'] as String,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'message_id': messageId,
      'channel_id': channelId,
      'sender_id': senderId,
      'content': content,
      'content_hash': contentHash,
      'created_at': createdAt,
      'on_chain_confirmed': onChainConfirmed,
    };
  }
}

/// Direct message with decrypted content
class DirectMessage {
  final String messageId;
  final String senderId;
  final String recipientId;
  final String content;
  final String contentHash;
  final String createdAt;
  final bool onChainConfirmed;

  DirectMessage({
    required this.messageId,
    required this.senderId,
    required this.recipientId,
    required this.content,
    required this.contentHash,
    required this.createdAt,
    required this.onChainConfirmed,
  });

  factory DirectMessage.fromJson(Map<String, dynamic> json) {
    return DirectMessage(
      messageId: json['message_id'] as String,
      senderId: json['sender_id'] as String,
      recipientId: json['recipient_id'] as String,
      content: json['content'] as String,
      contentHash: json['content_hash'] as String,
      createdAt: json['created_at'] as String,
      onChainConfirmed: json['on_chain_confirmed'] as bool,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'message_id': messageId,
      'sender_id': senderId,
      'recipient_id': recipientId,
      'content': content,
      'content_hash': contentHash,
      'created_at': createdAt,
      'on_chain_confirmed': onChainConfirmed,
    };
  }
}
