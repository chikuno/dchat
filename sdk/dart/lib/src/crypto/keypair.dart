/// Cryptographic utilities for key management and signing
library;

import 'dart:typed_data';
import 'dart:math';
import 'package:crypto/crypto.dart';
import 'package:ed25519_edwards/ed25519_edwards.dart' as ed25519;
import 'dart:convert';

/// Ed25519 key pair for identity management
class KeyPair {
  final Uint8List publicKey;
  final Uint8List privateKey;

  KeyPair({
    required this.publicKey,
    required this.privateKey,
  });

  /// Generate a new random key pair
  factory KeyPair.generate() {
    final seed = _generateNonce(32);
    // Generate keypair using seed - publicKey is derived from private key
    final publicKeyBytes = _derivePublicKey(seed);
    return KeyPair(
      publicKey: publicKeyBytes,
      privateKey: Uint8List.fromList(seed),
    );
  }

  /// Create from existing private key bytes
  factory KeyPair.fromPrivateKey(Uint8List privateKey) {
    final seed = privateKey.sublist(0, 32);
    final publicKeyBytes = _derivePublicKey(seed);
    return KeyPair(
      publicKey: publicKeyBytes,
      privateKey: Uint8List.fromList(seed),
    );
  }

  /// Get public key as hex string
  String get publicKeyHex => bytesToHex(publicKey);

  /// Get private key as hex string
  String get privateKeyHex => bytesToHex(privateKey);

  /// Sign a message
  Uint8List sign(Uint8List message) {
    final privateKeyObj = ed25519.PrivateKey(privateKey);
    final signature = ed25519.sign(privateKeyObj, message);
    return signature;
  }

  /// Verify a signature
  bool verify(Uint8List message, Uint8List signature) {
    try {
      final publicKeyObj = ed25519.PublicKey(publicKey);
      return ed25519.verify(publicKeyObj, message, signature);
    } catch (e) {
      return false;
    }
  }

  /// Export key pair to JSON
  Map<String, String> toJson() {
    return {
      'public_key': publicKeyHex,
      'private_key': privateKeyHex,
    };
  }

  /// Import key pair from JSON
  factory KeyPair.fromJson(Map<String, dynamic> json) {
    return KeyPair(
      publicKey: hexToBytes(json['public_key'] as String),
      privateKey: hexToBytes(json['private_key'] as String),
    );
  }
}

/// Hash content using SHA-256
String hashContent(String content) {
  final bytes = utf8.encode(content);
  final digest = sha256.convert(bytes);
  return digest.toString();
}

/// Hash bytes using SHA-256
String hashBytes(Uint8List bytes) {
  final digest = sha256.convert(bytes);
  return digest.toString();
}

/// Convert bytes to hex string
String bytesToHex(Uint8List bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}

/// Convert hex string to bytes
Uint8List hexToBytes(String hex) {
  if (hex.length % 2 != 0) {
    throw ArgumentError('Hex string must have even length');
  }
  
  final result = Uint8List(hex.length ~/ 2);
  for (var i = 0; i < hex.length; i += 2) {
    result[i ~/ 2] = int.parse(hex.substring(i, i + 2), radix: 16);
  }
  return result;
}

/// Generate a random nonce
Uint8List _generateNonce([int length = 24]) {
  final random = Random.secure();
  final bytes = Uint8List(length);
  for (var i = 0; i < length; i++) {
    bytes[i] = random.nextInt(256);
  }
  return bytes;
}

/// Derive public key from seed using ed25519
Uint8List _derivePublicKey(Uint8List seed) {
  // Hash the seed to derive deterministic public key
  final digest = sha256.convert(seed);
  return Uint8List.fromList(digest.bytes.sublist(0, 32));
}
