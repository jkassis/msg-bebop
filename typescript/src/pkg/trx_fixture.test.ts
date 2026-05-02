import fs from 'node:fs'
import path from 'node:path'
import { decodeMsg, type MsgWire } from './trx'

interface FixtureSuite {
  message_decode: Array<{
    name: string
    input: MsgWire
    expected: {
      version: number
      body_base64: string
    }
  }>
}

describe('trx conformance fixture', () => {
  it('decodes canonical trx fixture cases', () => {
    const fixturePath = path.resolve(__dirname, '../../../conformance/fixtures/trx_suite.v1.json')
    const suite = JSON.parse(fs.readFileSync(fixturePath, 'utf8')) as FixtureSuite

    for (const c of suite.message_decode) {
      const msg = decodeMsg(c.input)
      expect(msg.version).toBe(c.expected.version)
      expect(Buffer.from(msg.body).toString('base64')).toBe(c.expected.body_base64)
    }
  })
})
