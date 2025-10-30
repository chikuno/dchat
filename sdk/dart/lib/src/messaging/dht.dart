// Distributed Hash Table (DHT) for peer discovery and routing

/// DHT node for peer discovery
class DHTNode {
  final String nodeId;
  final String peerId;
  final String address;
  final int port;
  DateTime? lastSeen;

  DHTNode({
    required this.nodeId,
    required this.peerId,
    required this.address,
    required this.port,
  }) {
    lastSeen = DateTime.now();
  }

  bool isAlive({Duration timeout = const Duration(minutes: 10)}) {
    if (lastSeen == null) return false;
    return DateTime.now().difference(lastSeen!).inSeconds < timeout.inSeconds;
  }

  void updateLastSeen() => lastSeen = DateTime.now();

  Map<String, dynamic> toJson() => {
        'nodeId': nodeId,
        'peerId': peerId,
        'address': address,
        'port': port,
        'lastSeen': lastSeen?.toIso8601String(),
      };
}

/// Kademlia-like DHT implementation
class DHT {
  final String localNodeId;
  final Map<String, DHTNode> routingTable = {};
  static const int k = 20; // bucket size
  static const int maxBuckets = 160; // 160 bits for 256-bit keyspace

  DHT({required this.localNodeId});

  /// Calculate XOR distance between two node IDs
  int _xorDistance(String id1, String id2) {
    int distance = 0;
    for (int i = 0; i < id1.length && i < id2.length; i++) {
      distance ^= (id1.codeUnitAt(i) ^ id2.codeUnitAt(i));
    }
    return distance;
  }

  /// Find closest nodes to a target ID
  List<DHTNode> findClosest(String targetId, {int count = k}) {
    final nodes = routingTable.values.toList();
    nodes.sort((a, b) {
      final distA = _xorDistance(a.nodeId, targetId);
      final distB = _xorDistance(b.nodeId, targetId);
      return distA.compareTo(distB);
    });
    return nodes.take(count).toList();
  }

  /// Add node to routing table
  void addNode(DHTNode node) {
    if (node.nodeId == localNodeId) return; // Don't add self
    routingTable[node.nodeId] = node;
  }

  /// Remove dead nodes
  void pruneStalePeers({Duration timeout = const Duration(minutes: 10)}) {
    routingTable.removeWhere((_, node) => !node.isAlive(timeout: timeout));
  }

  /// Get all active nodes
  List<DHTNode> getActivePeers() {
    pruneStalePeers();
    return routingTable.values.toList();
  }

  /// Route message to closest peer for a target
  DHTNode? routeToClosest(String targetId) {
    final closest = findClosest(targetId, count: 1);
    return closest.isNotEmpty ? closest.first : null;
  }

  /// Lookup peer by ID
  DHTNode? lookup(String peerId) {
    final matching = routingTable.values
        .where((node) => node.peerId == peerId)
        .toList();
    return matching.isNotEmpty ? matching.first : null;
  }

  /// Get routing table stats
  Map<String, dynamic> getStats() => {
        'totalNodes': routingTable.length,
        'activePeers': getActivePeers().length,
        'stalePeers': routingTable.length - getActivePeers().length,
      };
}

/// Routing path for message delivery
class RoutingPath {
  final List<DHTNode> hops;
  final String targetId;
  final DateTime createdAt;

  RoutingPath({
    required this.hops,
    required this.targetId,
  }) : createdAt = DateTime.now();

  bool isValid({Duration ttl = const Duration(minutes: 5)}) {
    return DateTime.now().difference(createdAt).inSeconds < ttl.inSeconds;
  }

  int get hopCount => hops.length;

  List<String> get hopIds => hops.map((h) => h.nodeId).toList();
}
