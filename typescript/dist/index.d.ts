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
export * from './msg';
export declare class MsgUtils {
    /**
     * Create a new message with timestamp
     */
    static createWithTimestamp(body: string, fromId: string, toIds: string[], type: string): {
        msg: import('./msg').Msg;
        timestamp: number;
    };
    /**
     * Validate message structure
     */
    static validate(msg: import('./msg').Msg): boolean;
    /**
     * Calculate message size in bytes
     */
    static getSize(msg: import('./msg').Msg): number;
}
//# sourceMappingURL=index.d.ts.map