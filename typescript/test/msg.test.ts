import { Msg, MsgUtils } from '../src';

describe('Msg Bebop Library', () => {
  test('basic serialization', () => {
    const original = Msg({
      body: "Hello from TypeScript!",
      fromId: "ts_test",
      id: "test_001",
      toIds: ["user1", "user2"],
      type: "test"
    });

    // Use static method instead of instance method
    const bytes = Msg.encode(original);
    expect(bytes.length).toBeGreaterThan(0);

    const decoded = Msg.decode(bytes);
    expect(decoded.body).toBe(original.body);
    expect(decoded.fromId).toBe(original.fromId);
    expect(decoded.id).toBe(original.id);
    expect(decoded.toIds).toEqual(original.toIds);
    expect(decoded.type).toBe(original.type);
  });

  test('message utilities', () => {
    const { msg, timestamp } = MsgUtils.createWithTimestamp(
      "Test message",
      "sender",
      ["recipient"],
      "utility_test"
    );

    expect(MsgUtils.validate(msg)).toBe(true);
    expect(MsgUtils.getSize(msg)).toBeGreaterThan(0);
    expect(timestamp).toBeCloseTo(Date.now(), 2); // Within reasonable time
  });
});
