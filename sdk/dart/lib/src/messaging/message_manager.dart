// High-level message management with P2P routing and delivery tracking

import 'dart:async';
import 'package:uuid/uuid.dart';

import 'crypto.dart';
import 'dht.dart';
import 'peer_manager.dart';
import 'proof_of_delivery.dart';

/// P2P Message with metadata
class P2PMessage {
  final String messageId;
  final String senderId;
  final String recipientId;
  final String content;
  final DateTime timestamp;
  final String? routingPath;
  final bool isEncrypted;
  final Map<String, dynamic>? metadata;

  P2PMessage({
    required this.senderId,
    required this.recipientId,
    required this.content,
    this.routingPath,
    this.isEncrypted = true,
    this.metadata,
  })  : messageId = const Uuid().v4(),
        timestamp = DateTime.now();

  Map<String, dynamic> toJson() => {
        'messageId': messageId,
        'senderId': senderId,
        'recipientId': recipientId,
        'content': content,
        'timestamp': timestamp.toIso8601String(),
        'routingPath': routingPath,
        'isEncrypted': isEncrypted,
        'metadata': metadata,
      };

  factory P2PMessage.fromJson(Map<String, dynamic> json) {
    return P2PMessage(
      senderId: json['senderId'],
      recipientId: json['recipientId'],
      content: json['content'],
      routingPath: json['routingPath'],
      isEncrypted: json['isEncrypted'] ?? true,
      metadata: json['metadata'],
    );
  }
}

/// High-level message manager
class MessageManager {
  final String localPeerId;
  final String localPublicKey;
  final PeerManager peerManager;
  final DHT dht;
  final MessageCrypto crypto;
  final ProofOfDeliveryTracker deliveryTracker;

  final Map<String, P2PMessage> messageCache = {};
  final List<Function(P2PMessage)> onMessageReceived = [];
  final List<Function(DeliveryProof)> onDeliveryProof = [];

  MessageManager({
    required this.localPeerId,
    required this.localPublicKey,
    required this.peerManager,
    required this.dht,
    MessageCrypto? crypto,
    ProofOfDeliveryTracker? deliveryTracker,
  })  : crypto = crypto ?? MessageCrypto(),
        deliveryTracker = deliveryTracker ?? ProofOfDeliveryTracker();

  /// Send message to peer
  Future<String> sendMessage(
    String recipientId,
    String content, {
    bool encrypt = true,
  }) async {
    final message = P2PMessage(
      senderId: localPeerId,
      recipientId: recipientId,
      content: content,
      isEncrypted: encrypt,
    );

    // Encrypt if needed
    if (encrypt) {
      final recipientPeer = peerManager.getPeer(recipientId);
      if (recipientPeer != null) {
        message.content;
        // In production: crypto.encrypt(content, additionalData: recipientId)
      }
    }

    // Mark as pending delivery
    deliveryTracker.markPending(message.messageId, recipientId);
    messageCache[message.messageId] = message;

    // Find route to recipient
    final route = _findRoute(recipientId);
    if (route != null) {
      message.routingPath;
      // In production: send via route
    }

    return message.messageId;
  }

  /// Handle incoming message
  void handleIncomingMessage(Map<String, dynamic> messageJson) {
    try {
      final message = P2PMessage.fromJson(messageJson);

      // Decrypt if needed
      if (message.isEncrypted) {
        // In production: message.content = crypto.decrypt(...)
      }

      messageCache[message.messageId] = message;
      peerManager
          .getPeer(message.senderId)
          ?.recordMessage(message.content.length, incoming: true);

      // Notify listeners
      for (final listener in onMessageReceived) {
        listener(message);
      }
    } catch (e) {
      print('Error handling incoming message: $e');
    }
  }

  /// Handle delivery proof
  void handleDeliveryProof(Map<String, dynamic> proofJson) {
    try {
      final proof = DeliveryProof.fromJson(proofJson);
      deliveryTracker.recordProof(proof);

      // Notify listeners
      for (final listener in onDeliveryProof) {
        listener(proof);
      }
    } catch (e) {
      print('Error handling delivery proof: $e');
    }
  }

  /// Find optimal route to recipient
  RoutingPath? _findRoute(String recipientId) {
    // Check if recipient is direct peer
    if (peerManager.getPeer(recipientId) != null) {
      final peer = peerManager.getPeer(recipientId)!;
      if (peer.state == PeerState.connected) {
        return RoutingPath(
          hops: [
            DHTNode(
              nodeId: peer.peerId,
              peerId: peer.peerId,
              address: peer.address ?? 'unknown',
              port: peer.port ?? 0,
            )
          ],
          targetId: recipientId,
        );
      }
    }

    // Use DHT to find route
    final route = dht.findClosest(recipientId, count: 5);
    if (route.isNotEmpty) {
      return RoutingPath(
        hops: route,
        targetId: recipientId,
      );
    }

    return null;
  }

  /// Get message by ID
  P2PMessage? getMessage(String messageId) => messageCache[messageId];

  /// Get all messages from sender
  List<P2PMessage> getMessagesFrom(String senderId) {
    return messageCache.values
        .where((m) => m.senderId == senderId)
        .toList();
  }

  /// Get all messages to recipient
  List<P2PMessage> getMessagesTo(String recipientId) {
    return messageCache.values
        .where((m) => m.recipientId == recipientId)
        .toList();
  }

  /// Get messaging statistics
  Map<String, dynamic> getStats() {
    return {
      'cachedMessages': messageCache.length,
      'peers': peerManager.getStats(),
      'delivery': deliveryTracker.getStats(),
      'dht': dht.getStats(),
    };
  }

  /// Clean up old messages and stale peers
  void cleanup({Duration retention = const Duration(days: 7)}) {
    final cutoff = DateTime.now().subtract(retention);
    messageCache.removeWhere((_, msg) => msg.timestamp.isBefore(cutoff));
    peerManager.pruneStalepeers();
    deliveryTracker.prune(retention: retention);
  }
}

extension on PeerState {
  // For import reference
}