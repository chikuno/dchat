// Peer connection management for P2P messaging

/// Peer connection state
enum PeerState {
  unknown,
  connecting,
  connected,
  disconnected,
}

/// Peer information
class Peer {
  final String peerId;
  final String publicKey;
  late PeerState state;
  late DateTime connectedAt;
  late DateTime lastMessageTime;
  int messageCount = 0;
  int totalBytesSent = 0;
  int totalBytesReceived = 0;
  String? address;
  int? port;
  double? trustScore;

  Peer({
    required this.peerId,
    required this.publicKey,
    this.address,
    this.port,
  }) {
    state = PeerState.unknown;
    connectedAt = DateTime.now();
    lastMessageTime = DateTime.now();
    trustScore = 50.0; // Initial trust score 0-100
  }

  void connect() {
    state = PeerState.connecting;
  }

  void markConnected() {
    state = PeerState.connected;
    connectedAt = DateTime.now();
  }

  void disconnect() {
    state = PeerState.disconnected;
  }

  void recordMessage(int byteSize, {bool incoming = false}) {
    messageCount++;
    lastMessageTime = DateTime.now();
    if (incoming) {
      totalBytesReceived += byteSize;
    } else {
      totalBytesSent += byteSize;
    }
    _updateTrustScore();
  }

  void _updateTrustScore() {
    // Trust score increases with successful message delivery
    if (messageCount > 0 && state == PeerState.connected) {
      trustScore = (trustScore! + 0.5).clamp(0, 100);
    }
  }

  bool isStale({Duration timeout = const Duration(minutes: 10)}) {
    return DateTime.now().difference(lastMessageTime).inSeconds >
        timeout.inSeconds;
  }

  Map<String, dynamic> toJson() => {
        'peerId': peerId,
        'publicKey': publicKey,
        'state': state.toString(),
        'connectedAt': connectedAt.toIso8601String(),
        'lastMessageTime': lastMessageTime.toIso8601String(),
        'messageCount': messageCount,
        'totalBytesSent': totalBytesSent,
        'totalBytesReceived': totalBytesReceived,
        'address': address,
        'port': port,
        'trustScore': trustScore,
      };
}

/// Peer manager for managing P2P connections
class PeerManager {
  final String localPeerId;
  final Map<String, Peer> peers = {};
  final List<String> blockedPeers = [];
  int maxPeers = 100;

  PeerManager({required this.localPeerId});

  /// Add or get peer
  Peer addPeer(String peerId, String publicKey,
      {String? address, int? port}) {
    if (blockedPeers.contains(peerId)) {
      throw Exception('Peer is blocked');
    }

    if (peers.containsKey(peerId)) {
      return peers[peerId]!;
    }

    if (peers.length >= maxPeers) {
      _evictLeastTrustedPeer();
    }

    final peer = Peer(
      peerId: peerId,
      publicKey: publicKey,
      address: address,
      port: port,
    );
    peers[peerId] = peer;
    return peer;
  }

  /// Get peer by ID
  Peer? getPeer(String peerId) => peers[peerId];

  /// Remove peer
  void removePeer(String peerId) {
    peers.remove(peerId);
  }

  /// Block a peer
  void blockPeer(String peerId) {
    blockedPeers.add(peerId);
    removePeer(peerId);
  }

  /// Unblock a peer
  void unblockPeer(String peerId) {
    blockedPeers.remove(peerId);
  }

  /// Get all connected peers
  List<Peer> getConnectedPeers() {
    return peers.values
        .where((p) => p.state == PeerState.connected)
        .toList();
  }

  /// Get all active peers (connected or recently connected)
  List<Peer> getActivePeers({Duration recentThreshold = const Duration(minutes: 5)}) {
    final cutoff = DateTime.now().subtract(recentThreshold);
    return peers.values
        .where((p) =>
            p.state == PeerState.connected || p.lastMessageTime.isAfter(cutoff))
        .toList();
  }

  /// Clean up stale peers
  void pruneStalepeers({Duration timeout = const Duration(minutes: 30)}) {
    final stalePeers = peers.values
        .where((p) => p.isStale(timeout: timeout))
        .map((p) => p.peerId)
        .toList();

    for (final peerId in stalePeers) {
      removePeer(peerId);
    }
  }

  /// Evict least trusted peer
  void _evictLeastTrustedPeer() {
    if (peers.isEmpty) return;

    Peer? leastTrusted;
    double? minTrust;

    for (final peer in peers.values) {
      if (peer.state != PeerState.connected) {
        leastTrusted = peer;
        break;
      }
      if (minTrust == null || (peer.trustScore ?? 0) < minTrust) {
        minTrust = peer.trustScore ?? 0;
        leastTrusted = peer;
      }
    }

    if (leastTrusted != null) {
      removePeer(leastTrusted.peerId);
    }
  }

  /// Get peer statistics
  Map<String, dynamic> getStats() {
    final connected = getConnectedPeers();
    final active = getActivePeers();

    double avgTrustScore = 0;
    if (peers.isNotEmpty) {
      avgTrustScore = peers.values
              .map((p) => p.trustScore ?? 0)
              .reduce((a, b) => a + b) /
          peers.length;
    }

    return {
      'totalPeers': peers.length,
      'connectedPeers': connected.length,
      'activePeers': active.length,
      'blockedPeers': blockedPeers.length,
      'averageTrustScore': avgTrustScore.toStringAsFixed(2),
      'utilizationRate': ((connected.length / maxPeers) * 100).toStringAsFixed(2),
    };
  }
}
