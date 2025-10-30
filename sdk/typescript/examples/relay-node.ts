/**
 * Relay node example using dchat TypeScript SDK
 * 
 * Run with: npm run build && node dist/examples/relay-node.js
 */

import { RelayNode } from '../src';

async function main() {
  console.log('🚀 dchat SDK Relay Node Example\n');

  // Create a relay with custom configuration
  const relay = RelayNode.withConfig({
    name: 'MyRelayNode',
    listenAddr: '0.0.0.0',
    listenPort: 9000,
    stakingEnabled: true,
    minUptimePercent: 99.0,
  });

  const config = relay.getConfig();
  console.log(`✅ Relay node created: ${config.name}`);
  console.log(`📍 Listening on: ${config.listenAddr}:${config.listenPort}\n`);

  // Start the relay
  console.log('🌐 Starting relay node...');
  await relay.start();
  console.log('✅ Relay is running!\n');

  // Simulate running for a bit
  console.log('⏳ Running for 5 seconds...');
  await new Promise(resolve => setTimeout(resolve, 5000));

  // Get statistics
  console.log('\n📊 Relay Statistics:');
  const stats = await relay.getStats();
  console.log(`  Connected peers: ${stats.connectedPeers}`);
  console.log(`  Messages relayed: ${stats.messagesRelayed}`);
  console.log(`  Uptime: ${stats.uptimePercent.toFixed(2)}%`);
  console.log(`  Reputation score: ${stats.reputationScore}\n`);

  // Stop the relay
  console.log('🛑 Stopping relay node...');
  await relay.stop();
  console.log('✅ Relay stopped cleanly\n');

  console.log('🎉 Example completed successfully!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
