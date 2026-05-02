export interface MsgWire {
  id: string
  from_id: string
  to_ids: string[]
  type_: string
  version?: number
  body: string
}

export interface Msg {
  id: string
  from_id: string
  to_ids: string[]
  type_: string
  version: number
  body: Uint8Array
}

export function createMsg(input: Omit<Msg, 'version'> & { version?: number }): Msg {
  return {
    ...input,
    version: input.version ?? 1,
  }
}

export function encodeMsg(msg: Msg): MsgWire {
  return {
    id: msg.id,
    from_id: msg.from_id,
    to_ids: msg.to_ids,
    type_: msg.type_,
    version: msg.version,
    body: Buffer.from(msg.body).toString('base64'),
  }
}

export function decodeMsg(wire: MsgWire): Msg {
  return {
    id: wire.id,
    from_id: wire.from_id,
    to_ids: wire.to_ids,
    type_: wire.type_,
    version: wire.version ?? 1,
    body: new Uint8Array(Buffer.from(wire.body, 'base64')),
  }
}
