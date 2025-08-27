from enum import Enum
from python_bebop import BebopWriter, BebopReader, UnionType, UnionDefinition
from uuid import UUID
import math
import json
from datetime import datetime

class Msg:
    _body: str

    _fromId: str

    _id: str

    _toIds: list[str]

    _type: str


    def __init__(self,     body: str, fromId: str, id: str, toIds: list[str], type: str    ):
        self.encode = self._encode
        self._body = body
        self._fromId = fromId
        self._id = id
        self._toIds = toIds
        self._type = type

    @property
    def body(self):
        return self._body

    @property
    def fromId(self):
        return self._fromId

    @property
    def id(self):
        return self._id

    @property
    def toIds(self):
        return self._toIds

    @property
    def type(self):
        return self._type

    def _encode(self):
        """Fake class method for allowing instance encode"""
        writer = BebopWriter()
        Msg.encode_into(self, writer)
        return writer.to_list()


    @staticmethod
    def encode(message: "Msg"):
        writer = BebopWriter()
        Msg.encode_into(message, writer)
        return writer.to_list()


    @staticmethod
    def encode_into(message: "Msg", writer: BebopWriter):
        writer.write_string(message.body)

        writer.write_string(message.fromId)

        writer.write_string(message.id)

        length0 = len(message.toIds)
        writer.write_uint32(length0)
        for i0 in range(length0):
            writer.write_string(message.toIds[i0])

        writer.write_string(message.type)

    @classmethod
    def read_from(cls, reader: BebopReader):
        field0 = reader.read_string()

        field1 = reader.read_string()

        field2 = reader.read_string()

        length0 = reader.read_uint32()
        field3 = []
        for i0 in range(length0):
            x0 = reader.read_string()
            field3.append(x0)

        field4 = reader.read_string()

        return Msg(body=field0, fromId=field1, id=field2, toIds=field3, type=field4)

    @staticmethod
    def decode(buffer) -> "Msg":
        return Msg.read_from(BebopReader(buffer))

    def __repr__(self):
        return json.dumps(self, default=lambda o: o.value if isinstance(o, Enum) else dict(sorted(o.__dict__.items())) if hasattr(o, "__dict__") else str(o))



