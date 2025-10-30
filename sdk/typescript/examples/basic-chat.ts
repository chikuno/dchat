/**
 * Basic chat example using dchat TypeScript SDK
 * 
 * Run with: npm run build && node dist/examples/basic-chat.js
 */

import { Client } from '../src';

async function main() {
  console.log('🚀 dchat SDK Basic Chat Example\n');

  // Create a client with the builder pattern
  const alice = await Client.builder()
    .name('Alice')
    .dataDir('/tmp/dchat_alice')
    .listenPort(9001)
    .encryption(true)
    .build();

  const identity = alice.getIdentity();
  console.log(`✅ Client created for: ${identity.username}`);
  console.log(`📍 User ID: ${identity.userId}`);
  console.log(`🔑 Public key: ${identity.publicKey}\n`);

  // Connect to the network
  console.log('🌐 Connecting to dchat network...');
  await alice.connect();
  console.log('✅ Connected!\n');

  // Send a message
  console.log('📤 Sending message...');
  await alice.sendMessage('Hello, decentralized world!');
  console.log('✅ Message sent!\n');

  // Receive messages
  console.log('📥 Fetching messages...');
  const messages = await alice.receiveMessages();
  console.log(`✅ Received ${messages.length} message(s)\n`);

  for (let i = 0; i < messages.length; i++) {
    const msg = messages[i];
    if (msg.content.type === 'Text') {
      console.log(`Message #${i + 1}: ${msg.content.text}`);
    }
  }

  // Disconnect
  console.log('\n🔌 Disconnecting...');
  await alice.disconnect();
  console.log('✅ Disconnected cleanly\n');

  console.log('🎉 Example completed successfully!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
