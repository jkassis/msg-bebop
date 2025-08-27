/**
 * Msg Bebop Library for TypeScript/JavaScript
 *
 * High-performance message serialization using Bebop.
 *
 * @example
 * ```typescript
 * import { Msg } from 'msg';
 *
 * const msg = Msg({
 *   body: "Hello, world!",
 *   fromId: "sender123",
 *   id: "msg456",
 *   toIds: ["recipient1", "recipient2"],
 *   type: "greeting"
 * });
 *
 * // Serialize
 * const bytes = msg.encode();
 *
 * // Deserialize
 * const decodedMsg = Msg.decode(bytes);
 * ```
 */

// Re-export generated Bebop types
export * from './msg';
import { Msg } from './msg';  // Import Msg for use in utilities

// Additional utility functions
export class MsgUtils {
  /**
   * Create a new message with timestamp
   */
  static createWithTimestamp(
    body: string,
    fromId: string,
    toIds: string[],
    type: string
  ): { msg: import('./msg').Msg; timestamp: number } {
    const timestamp = Date.now();
    const id = `${fromId}-${timestamp}-${Math.random().toString(36).substring(2)}`;

    const msg = Msg({
      body,
      fromId,
      id,
      toIds,
      type
    });

    return { msg, timestamp };
  }

  /**
   * Validate message structure
   */
  static validate(msg: import('./msg').Msg): boolean {
    return !!(
      msg.body &&
      msg.fromId &&
      msg.id &&
      Array.isArray(msg.toIds) &&
      msg.type
    );
  }

  /**
   * Calculate message size in bytes
   */
  static getSize(msg: import('./msg').Msg): number {
    return Msg.encode(msg).length;  // Use static method instead
  }
}
