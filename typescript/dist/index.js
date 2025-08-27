"use strict";
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
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MsgUtils = void 0;
// Re-export generated Bebop types
__exportStar(require("./msg"), exports);
const msg_1 = require("./msg"); // Import Msg for use in utilities
// Additional utility functions
class MsgUtils {
    /**
     * Create a new message with timestamp
     */
    static createWithTimestamp(body, fromId, toIds, type) {
        const timestamp = Date.now();
        const id = `${fromId}-${timestamp}-${Math.random().toString(36).substring(2)}`;
        const msg = (0, msg_1.Msg)({
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
    static validate(msg) {
        return !!(msg.body &&
            msg.fromId &&
            msg.id &&
            Array.isArray(msg.toIds) &&
            msg.type);
    }
    /**
     * Calculate message size in bytes
     */
    static getSize(msg) {
        return msg_1.Msg.encode(msg).length; // Use static method instead
    }
}
exports.MsgUtils = MsgUtils;
//# sourceMappingURL=index.js.map