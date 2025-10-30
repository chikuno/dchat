/**
 * dchat SDK error types
 */

export enum ErrorCode {
  CONFIG = 'CONFIG',
  NETWORK = 'NETWORK',
  CRYPTO = 'CRYPTO',
  STORAGE = 'STORAGE',
  IDENTITY = 'IDENTITY',
  MESSAGE = 'MESSAGE',
  NOT_CONNECTED = 'NOT_CONNECTED',
  ALREADY_CONNECTED = 'ALREADY_CONNECTED',
  TIMEOUT = 'TIMEOUT',
  UNKNOWN = 'UNKNOWN',
}

export class SdkError extends Error {
  public readonly code: ErrorCode;
  public readonly details?: unknown;

  constructor(code: ErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = 'SdkError';
    this.code = code;
    this.details = details;
    Object.setPrototypeOf(this, SdkError.prototype);
  }

  static config(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.CONFIG, message, details);
  }

  static network(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.NETWORK, message, details);
  }

  static crypto(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.CRYPTO, message, details);
  }

  static storage(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.STORAGE, message, details);
  }

  static identity(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.IDENTITY, message, details);
  }

  static message(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.MESSAGE, message, details);
  }

  static notConnected(): SdkError {
    return new SdkError(ErrorCode.NOT_CONNECTED, 'Client is not connected to the network');
  }

  static alreadyConnected(): SdkError {
    return new SdkError(ErrorCode.ALREADY_CONNECTED, 'Client is already connected');
  }

  static timeout(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.TIMEOUT, message, details);
  }

  static unknown(message: string, details?: unknown): SdkError {
    return new SdkError(ErrorCode.UNKNOWN, message, details);
  }
}
