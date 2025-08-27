"""
Msg Bebop Library for Python

High-performance message serialization using Bebop.

Example:
    from msg import Msg

    msg = Msg(
        body="Hello, world!",
        from_id="sender123",
        id="msg456",
        to_ids=["recipient1", "recipient2"],
        type_="greeting"
    )

    # Serialize
    bytes_data = msg.encode()

    # Deserialize
    decoded_msg = Msg.decode(bytes_data)
"""

from .msg import *

__version__ = "0.1.0"
__all__ = ["Msg", "MsgUtils"]

import time
import uuid
from typing import List, Optional, Dict, Any


class MsgUtils:
    """Utility functions for Msg handling"""

    @staticmethod
    def create_with_timestamp(
        body: str,
        from_id: str,
        to_ids: List[str],
        type_: str
    ) -> tuple[Msg, float]:
        """Create a new message with timestamp"""
        timestamp = time.time()
        msg_id = f"{from_id}-{int(timestamp * 1000)}-{uuid.uuid4().hex[:8]}"

        msg = Msg(
            body=body,
            from_id=from_id,
            id=msg_id,
            to_ids=to_ids,
            type_=type_
        )

        return msg, timestamp

    @staticmethod
    def validate(msg: Msg) -> bool:
        """Validate message structure"""
        return all([
            msg.body,
            msg.from_id,
            msg.id,
            isinstance(msg.to_ids, list),
            msg.type_
        ])

    @staticmethod
    def get_size(msg: Msg) -> int:
        """Calculate message size in bytes"""
        return len(msg.encode())

    @staticmethod
    def to_dict(msg: Msg) -> Dict[str, Any]:
        """Convert message to dictionary"""
        return {
            "body": msg.body,
            "from_id": msg.from_id,
            "id": msg.id,
            "to_ids": msg.to_ids,
            "type_": msg.type_
        }

    @staticmethod
    def from_dict(data: Dict[str, Any]) -> Msg:
        """Create message from dictionary"""
        return Msg(
            body=data["body"],
            from_id=data["from_id"],
            id=data["id"],
            to_ids=data["to_ids"],
            type_=data["type_"]
        )
