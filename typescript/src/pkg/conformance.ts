import fs from 'node:fs'

export interface RawMsg {
  body: string
  from_id: string
  id: string
  to_ids: string[]
  type_: string
  version?: number
  ack_msg_id?: string
  ack_from_id?: string
  ack_to_id?: string
  ack_version?: number
}

interface MessageDecodeCase {
  name: string
  input: RawMsg
  expected: { version: number }
}

interface TxValidateCase {
  name: string
  msg: RawMsg
  expected_error_code: string
}

interface AckExpected {
  deleted: boolean
  remaining_to_ids: string[]
}

interface AckApplyCase {
  name: string
  courier_id: string
  tx_msg: RawMsg
  ack_msg: RawMsg
  expected?: AckExpected
  expected_error_code?: string
}

interface Suite {
  message_decode: MessageDecodeCase[]
  tx_validate: TxValidateCase[]
  ack_apply: AckApplyCase[]
}

interface Msg {
  body: string
  from_id: string
  id: string
  to_ids: string[]
  type_: string
  version: number
  ack_msg_id?: string
  ack_from_id?: string
  ack_to_id?: string
  ack_version?: number
}

interface AckApplyResult {
  deleted: boolean
  remaining_to_ids: string[]
}

function decodeMsg(raw: RawMsg): Msg {
  return {
    ...raw,
    version: raw.version === undefined ? 1 : raw.version,
  }
}

function validateTxMsg(msg: Msg): string | null {
  if (msg.version === 0) {
    return 'invalid_msg_version'
  }
  const hasAckFields = msg.ack_msg_id !== undefined
    || msg.ack_from_id !== undefined
    || msg.ack_to_id !== undefined
    || msg.ack_version !== undefined
  if (msg.type_ !== 'Ack' && hasAckFields) {
    return 'non_ack_has_ack_fields'
  }
  return null
}

function applyAck(courierID: string, txMsg: Msg, ackMsg: Msg): AckApplyResult | string {
  if (ackMsg.ack_msg_id === undefined) return 'missing_ack_msg_id'
  if (ackMsg.ack_from_id === undefined) return 'missing_ack_from_id'
  if (ackMsg.ack_to_id === undefined) return 'missing_ack_to_id'
  if (ackMsg.ack_version === undefined) return 'missing_ack_version'
  if (ackMsg.from_id !== ackMsg.ack_from_id) return 'ack_from_mismatch'
  if (!ackMsg.to_ids.includes(ackMsg.ack_to_id)) return 'ack_to_missing_in_envelope'
  if (ackMsg.ack_to_id !== courierID) return 'ack_to_wrong_courier'
  if (ackMsg.version !== ackMsg.ack_version) return 'ack_envelope_version_mismatch'
  if (ackMsg.ack_msg_id !== txMsg.id) return 'ack_msg_id_mismatch'
  if (ackMsg.ack_version !== txMsg.version) return 'ack_version_mismatch'

  const remaining = txMsg.to_ids.filter((id) => id !== ackMsg.ack_from_id).sort()
  if (remaining.length === txMsg.to_ids.length) {
    return 'ack_from_not_pending'
  }
  return {
    deleted: remaining.length === 0,
    remaining_to_ids: remaining,
  }
}

export function runConformanceFixture(path: string): void {
  const raw = fs.readFileSync(path, 'utf8')
  const suite: Suite = JSON.parse(raw)

  for (const c of suite.message_decode) {
    const msg = decodeMsg(c.input)
    if (msg.version !== c.expected.version) {
      throw new Error(`case ${c.name} expected version ${c.expected.version} got ${msg.version}`)
    }
  }

  for (const c of suite.tx_validate) {
    const err = validateTxMsg(decodeMsg(c.msg))
    if (err === null) {
      throw new Error(`case ${c.name} expected tx validation error ${c.expected_error_code}`)
    }
    if (err !== c.expected_error_code) {
      throw new Error(`case ${c.name} expected tx error ${c.expected_error_code} got ${err}`)
    }
  }

  for (const c of suite.ack_apply) {
    const result = applyAck(c.courier_id, decodeMsg(c.tx_msg), decodeMsg(c.ack_msg))
    if (c.expected_error_code !== undefined) {
      if (typeof result !== 'string') {
        throw new Error(`case ${c.name} expected ack error ${c.expected_error_code}`)
      }
      if (result !== c.expected_error_code) {
        throw new Error(`case ${c.name} expected ack error ${c.expected_error_code} got ${result}`)
      }
      continue
    }
    if (typeof result === 'string') {
      throw new Error(`case ${c.name} unexpected ack error ${result}`)
    }
    if (!c.expected) {
      throw new Error(`case ${c.name} missing expected success payload`)
    }
    const got = [...result.remaining_to_ids].sort()
    const want = [...c.expected.remaining_to_ids].sort()
    if (result.deleted !== c.expected.deleted) {
      throw new Error(`case ${c.name} expected deleted=${c.expected.deleted} got ${result.deleted}`)
    }
    if (JSON.stringify(got) !== JSON.stringify(want)) {
      throw new Error(`case ${c.name} expected remaining_to_ids=${JSON.stringify(want)} got ${JSON.stringify(got)}`)
    }
  }
}
