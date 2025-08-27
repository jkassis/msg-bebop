import unittest
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from msg import Msg, MsgUtils


class TestMsgBebop(unittest.TestCase):

    def test_basic_serialization(self):
        original = Msg(
            body="Hello from Python!",
            from_id="python_test",
            id="test_001",
            to_ids=["user1", "user2"],
            type_="test"
        )

        # Serialize
        bytes_data = original.encode()
        self.assertGreater(len(bytes_data), 0)

        # Deserialize
        decoded = Msg.decode(bytes_data)

        # Verify
        self.assertEqual(decoded.body, original.body)
        self.assertEqual(decoded.from_id, original.from_id)
        self.assertEqual(decoded.id, original.id)
        self.assertEqual(decoded.to_ids, original.to_ids)
        self.assertEqual(decoded.type_, original.type_)

    def test_message_utils(self):
        msg, timestamp = MsgUtils.create_with_timestamp(
            "Test message",
            "sender",
            ["recipient"],
            "utility_test"
        )

        self.assertTrue(MsgUtils.validate(msg))
        self.assertGreater(MsgUtils.get_size(msg), 0)
        self.assertIsInstance(timestamp, float)

        # Test dictionary conversion
        msg_dict = MsgUtils.to_dict(msg)
        reconstructed = MsgUtils.from_dict(msg_dict)
        self.assertEqual(msg.body, reconstructed.body)


if __name__ == '__main__':
    unittest.main()
