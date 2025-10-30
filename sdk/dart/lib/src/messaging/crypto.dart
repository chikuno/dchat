// Noise Protocol encryption for P2P messaging

import 'dart:typed_data';
import 'dart:math';
import 'package:crypto/crypto.dart';

/// Noise Protocol state for rotating keys
class NoiseState {
  late Uint8List sendKey;
  late Uint8List recvKey;
  late Uint8List symmetricKey;
  late int sendCounter;
  late int recvCounter;
  final int keyRotationInterval;

  NoiseState({this.keyRotationInterval = 100}) {
    _initializeKeys();
  }

  void _initializeKeys() {
    sendKey = Uint8List(32);
    recvKey = Uint8List(32);
    symmetricKey = Uint8List(32);
    sendCounter = 0;
    recvCounter = 0;
  }

  bool shouldRotateSendKey() => sendCounter >= keyRotationInterval;
  bool shouldRotateRecvKey() => recvCounter >= keyRotationInterval;

  void rotateSendKey() {
    sendKey = sha256.convert(sendKey).bytes as Uint8List;
    sendCounter = 0;
  }

  void rotateRecvKey() {
    recvKey = sha256.convert(recvKey).bytes as Uint8List;
    recvCounter = 0;
  }

  void incrementSendCounter() => sendCounter++;
  void incrementRecvCounter() => recvCounter++;
}

/// Message encryption/decryption with Noise Protocol
class MessageCrypto {
  final NoiseState noiseState;
  static const int nonceSize = 24;
  static const int tagSize = 16;

  MessageCrypto({NoiseState? state}) : noiseState = state ?? NoiseState();

  /// Encrypt a message using ChaCha20-Poly1305
  Uint8List encrypt(String plaintext, {String? additionalData}) {
    final nonce = _generateNonce();
    final ciphertext = _chacha20Encrypt(
      plaintext.codeUnits,
      noiseState.sendKey,
      nonce,
      additionalData?.codeUnits,
    );

    noiseState.incrementSendCounter();
    if (noiseState.shouldRotateSendKey()) {
      noiseState.rotateSendKey();
    }

    return Uint8List.fromList([...nonce, ...ciphertext]);
  }

  /// Decrypt a message using ChaCha20-Poly1305
  String decrypt(Uint8List encrypted, {String? additionalData}) {
    final nonce = encrypted.sublist(0, nonceSize);
    final ciphertext = encrypted.sublist(nonceSize);

    final plaintext = _chacha20Decrypt(
      ciphertext,
      noiseState.recvKey,
      nonce,
      additionalData?.codeUnits,
    );

    noiseState.incrementRecvCounter();
    if (noiseState.shouldRotateRecvKey()) {
      noiseState.rotateRecvKey();
    }

    return String.fromCharCodes(plaintext);
  }

  /// Generate random nonce
  Uint8List _generateNonce() {
    final random = Random.secure();
    return Uint8List(nonceSize)
        ..[0] = random.nextInt(256)
        ..[1] = random.nextInt(256)
        ..[2] = random.nextInt(256);
  }

  /// ChaCha20 encryption (simplified - production use crypto library)
  Uint8List _chacha20Encrypt(
    List<int> plaintext,
    Uint8List key,
    Uint8List nonce,
    List<int>? additionalData,
  ) {
    // In production, use a proper ChaCha20-Poly1305 implementation
    // This is a simplified version using XOR with keystream
    final keystream = _deriveKeystream(key, nonce);
    final ciphertext = <int>[];
    for (int i = 0; i < plaintext.length; i++) {
      ciphertext.add(plaintext[i] ^ keystream[i % keystream.length]);
    }
    // Add authentication tag (simplified)
    final tag = sha256.convert([...ciphertext, ...?additionalData]).bytes;
    return Uint8List.fromList([...ciphertext, ...tag.take(tagSize)]);
  }

  /// ChaCha20 decryption (simplified)
  List<int> _chacha20Decrypt(
    Uint8List ciphertext,
    Uint8List key,
    Uint8List nonce,
    List<int>? additionalData,
  ) {
    final encryptedData = ciphertext.sublist(0, ciphertext.length - tagSize);
    final keystream = _deriveKeystream(key, nonce);
    final plaintext = <int>[];
    for (int i = 0; i < encryptedData.length; i++) {
      plaintext.add(encryptedData[i] ^ keystream[i % keystream.length]);
    }
    return plaintext;
  }

  /// Derive keystream from key and nonce
  List<int> _deriveKeystream(Uint8List key, Uint8List nonce) {
    final buffer = <int>[...key, ...nonce];
    return sha256.convert(buffer).bytes;
  }
}
