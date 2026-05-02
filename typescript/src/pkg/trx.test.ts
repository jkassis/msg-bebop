import { createMsg, decodeMsg, encodeMsg } from './trx'

describe('trx msg', () => {
  it('encodes raw bytes as base64 in JSON-safe wire format', () => {
    const msg = createMsg({
      id: 'm1',
      from_id: 'sender',
      to_ids: ['receiver'],
      type_: 'event',
      body: new Uint8Array([0, 1, 2, 255]),
    })

    expect(encodeMsg(msg)).toEqual({
      id: 'm1',
      from_id: 'sender',
      to_ids: ['receiver'],
      type_: 'event',
      version: 1,
      body: 'AAEC/w==',
    })
  })

  it('defaults version to 1 when decoding', () => {
    const msg = decodeMsg({
      id: 'm1',
      from_id: 'sender',
      to_ids: ['receiver'],
      type_: 'event',
      body: 'aGk=',
    })

    expect(msg.version).toBe(1)
    expect(Buffer.from(msg.body).toString('utf8')).toBe('hi')
  })
})
