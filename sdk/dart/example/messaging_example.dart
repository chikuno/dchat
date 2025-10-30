/// Example: P2P Messaging with DHT Routing and Proof of Delivery
///
/// Demonstrates:
/// - Creating message managers
/// - Sending encrypted messages
/// - DHT peer discovery
/// - Proof of delivery tracking
/// - Peer trust management

import 'package:dchat_sdk/dchat.dart';

void main() async {
  print('=== dchat P2P Messaging Example ===\n');

  // Initialize local peer
  const localPeerId = 'peer-1-local-id';
  const localPublicKey = 'peer-1-public-key';

  // Create peer manager
  final peerManager = PeerManager(localPeerId: localPeerId);

  // Create DHT
  final dht = DHT(localNodeId: localPeerId);

  // Create message manager
  final messageManager = MessageManager(
    localPeerId: localPeerId,
    localPublicKey: localPublicKey,
    peerManager: peerManager,
    dht: dht,
  );

  print('✓ Message manager initialized');
  print('  Local peer ID: $localPeerId\n');

  // === Peer Discovery ===
  print('--- Adding Peers to DHT ---');

  // Add some peers
  final peer1 = peerManager.addPeer(
    'peer-2-id',
    'peer-2-public-key',
    address: '192.168.1.100',
    port: 5000,
  );
  peer1.markConnected();

  final peer2 = peerManager.addPeer(
    'peer-3-id',
    'peer-3-public-key',
    address: '192.168.1.101',
    port: 5001,
  );
  peer2.markConnected();

  final _ = peerManager.addPeer(
    'peer-4-id',
    'peer-4-public-key',
    address: '192.168.1.102',
    port: 5002,
  );

  print('✓ Added 3 peers to network');

  // Add peers to DHT
  for (final peer in peerManager.getConnectedPeers()) {
    final dhtNode = DHTNode(
      nodeId: peer.peerId,
      peerId: peer.peerId,
      address: peer.address ?? 'unknown',
      port: peer.port ?? 0,
    );
    dht.addNode(dhtNode);
  }

  print('✓ Peers added to DHT routing table\n');

  // === Peer Statistics ===
  print('--- Peer Statistics ---');
  final peerStats = peerManager.getStats();
  print('Total peers: ${peerStats['totalPeers']}');
  print('Connected peers: ${peerStats['connectedPeers']}');
  print('Average trust score: ${peerStats['averageTrustScore']}\n');

  // === DHT Statistics ===
  print('--- DHT Statistics ---');
  final dhtStats = dht.getStats();
  print('Total DHT nodes: ${dhtStats['totalNodes']}');
  print('Active peers: ${dhtStats['activePeers']}\n');

  // === Sending Messages ===
  print('--- Sending Messages ---');

  final msgId1 = await messageManager.sendMessage(
    'peer-2-id',
    'Hello, peer 2!',
    encrypt: true,
  );
  print('✓ Sent message to peer-2-id: $msgId1');

  final msgId2 = await messageManager.sendMessage(
    'peer-3-id',
    'Hi peer 3, how are you?',
    encrypt: true,
  );
  print('✓ Sent message to peer-3-id: $msgId2');

  final msgId3 = await messageManager.sendMessage(
    'peer-4-id',
    'Message to peer 4',
    encrypt: true,
  );
  print('✓ Sent message to peer-4-id: $msgId3\n');

  // === Proof of Delivery ===
  print('--- Delivery Proof Tracking ---');

  // Simulate delivery proofs
  final proof1 = DeliveryProof(
    messageId: msgId1,
    recipientId: 'peer-2-id',
    senderPublicKey: localPublicKey,
    status: DeliveryStatus.delivered,
    relayNodeId: 'relay-node-1',
    blockHeight: 12345,
  );
  messageManager.handleDeliveryProof(proof1.toJson());
  print('✓ Message $msgId1 delivered');

  final proof2 = DeliveryProof(
    messageId: msgId2,
    recipientId: 'peer-3-id',
    senderPublicKey: localPublicKey,
    status: DeliveryStatus.read,
    blockHeight: 12346,
  );
  messageManager.handleDeliveryProof(proof2.toJson());
  print('✓ Message $msgId2 read\n');

  // === Receiving Messages ===
  print('--- Simulating Incoming Messages ---');

  messageManager.onMessageReceived.add((message) {
    print(
      '✓ Received message from ${message.senderId}: "${message.content}"',
    );
  });

  final incomingMsg1 = P2PMessage(
    senderId: 'peer-2-id',
    recipientId: localPeerId,
    content: 'Hello back!',
  );
  messageManager.handleIncomingMessage(incomingMsg1.toJson());

  final incomingMsg2 = P2PMessage(
    senderId: 'peer-3-id',
    recipientId: localPeerId,
    content: 'Doing great, thanks!',
  );
  messageManager.handleIncomingMessage(incomingMsg2.toJson());
  print('');

  // === Message Cache Query ===
  print('--- Message Cache ---');
  final allMessages = messageManager.messageCache.values.toList();
  print('Total messages in cache: ${allMessages.length}');

  final peer2Messages = messageManager.getMessagesFrom('peer-2-id');
  print('Messages from peer-2-id: ${peer2Messages.length}\n');

  // === Delivery Statistics ===
  print('--- Delivery Statistics ---');
  final deliveryStats = messageManager.deliveryTracker.getStats();
  print('Total deliveries tracked: ${deliveryStats['total']}');
  print('Successfully delivered: ${deliveryStats['delivered']}');
  print('Read receipts: ${deliveryStats['read']}');
  print('Success rate: ${deliveryStats['successRate']}%\n');

  // === Message Manager Statistics ===
  print('--- Overall Messaging Statistics ---');
  final allStats = messageManager.getStats();
  print('Cached messages: ${allStats['cachedMessages']}');
  print('Peer stats: ${allStats['peers']}');
  print('Delivery stats: ${allStats['delivery']}');
  print('DHT stats: ${allStats['dht']}\n');

  // === Peer Trust Management ===
  print('--- Peer Trust Management ---');
  final connectedPeers = peerManager.getConnectedPeers();
  for (final peer in connectedPeers) {
    print('Peer ${peer.peerId}:');
    print('  State: ${peer.state}');
    print('  Messages: ${peer.messageCount}');
    print('  Trust score: ${peer.trustScore}');
  }
  print('');

  // === Blocking Peers ===
  print('--- Blocking Peers ---');
  peerManager.blockPeer('peer-4-id');
  print('✓ Blocked peer-4-id');
  print('Blocked peers: ${peerManager.blockedPeers.length}\n');

  // === Cleanup ===
  print('--- Cleanup ---');
  messageManager.cleanup(retention: Duration(days: 7));
  print('✓ Cleaned up old messages and stale peers');

  print('\n=== Example Complete ===');
}
