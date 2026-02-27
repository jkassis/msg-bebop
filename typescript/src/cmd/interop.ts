import net from 'node:net'

type Msg = {
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

type Envelope = {
  msg: Msg
  hops: string[]
}

function parseArgs(argv: string[]): Record<string, string | boolean> {
  const out: Record<string, string | boolean> = {}
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i]
    if (!a.startsWith('--')) continue
    const key = a.slice(2)
    const nxt = argv[i + 1]
    if (!nxt || nxt.startsWith('--')) {
      out[key] = true
    } else {
      out[key] = nxt
      i += 1
    }
  }
  return out
}

function readJSONLine(socket: net.Socket): Promise<Envelope> {
  return new Promise((resolve, reject) => {
    let buf = ''
    socket.setEncoding('utf8')
    socket.on('data', (chunk) => {
      buf += chunk
      const idx = buf.indexOf('\n')
      if (idx >= 0) {
        const line = buf.slice(0, idx)
        try {
          const parsed = JSON.parse(line) as Envelope
          resolve(parsed)
        } catch (err) {
          reject(err)
        }
      }
    })
    socket.on('error', reject)
    socket.on('end', () => {
      if (!buf.includes('\n')) reject(new Error('connection closed before payload'))
    })
  })
}

function writeJSONLine(socket: net.Socket, env: Envelope): Promise<void> {
  return new Promise((resolve, reject) => {
    socket.write(`${JSON.stringify(env)}\n`, (err) => {
      if (err) reject(err)
      else resolve()
    })
  })
}

async function forward(next: string, env: Envelope): Promise<Envelope> {
  const [host, portRaw] = next.split(':')
  const port = Number(portRaw)
  return new Promise((resolve, reject) => {
    const conn = net.createConnection({ host, port }, async () => {
      try {
        await writeJSONLine(conn, env)
        const resp = await readJSONLine(conn)
        conn.end()
        resolve(resp)
      } catch (err) {
        conn.destroy()
        reject(err)
      }
    })
    conn.on('error', reject)
  })
}

async function runServer(listen: string, node: string, next?: string, once = false): Promise<void> {
  const [host, portRaw] = listen.split(':')
  const port = Number(portRaw)
  const server = net.createServer((conn) => {
    void (async () => {
      try {
        const req = await readJSONLine(conn)
        req.hops.push(node)
        const resp = next ? await forward(next, req) : {
          msg: {
            body: req.msg.id,
            from_id: node,
            id: `ack-${req.msg.id}`,
            to_ids: [req.msg.from_id],
            type_: 'Ack',
            version: req.msg.version,
            ack_msg_id: req.msg.id,
            ack_from_id: node,
            ack_to_id: req.msg.from_id,
            ack_version: req.msg.version,
          },
          hops: req.hops,
        }
        await writeJSONLine(conn, resp)
        conn.end()
        if (once) {
          server.close()
        }
      } catch {
        conn.destroy()
        process.exitCode = 1
      }
    })()
  })

  await new Promise<void>((resolve, reject) => {
    server.listen(port, host, () => resolve())
    server.on('error', reject)
    server.on('close', () => resolve())
  })
}

async function runClient(addr: string, node: string, expectHops: string[], expectAckFrom: string): Promise<void> {
  const [host, portRaw] = addr.split(':')
  const port = Number(portRaw)
  await new Promise<void>((resolve, reject) => {
    const conn = net.createConnection({ host, port }, async () => {
      try {
        const req: Envelope = {
          msg: {
            body: 'interop',
            from_id: node,
            id: 'interop-msg',
            to_ids: ['receiver'],
            type_: 'text',
            version: 1,
          },
          hops: [node],
        }
        await writeJSONLine(conn, req)
        const resp = await readJSONLine(conn)
        conn.end()
        if (JSON.stringify(resp.hops) !== JSON.stringify(expectHops)) {
          reject(new Error(`unexpected hops: got ${JSON.stringify(resp.hops)} want ${JSON.stringify(expectHops)}`))
          return
        }
        if (resp.msg.type_ !== 'Ack') {
          reject(new Error(`expected Ack response, got ${resp.msg.type_}`))
          return
        }
        if (resp.msg.ack_msg_id !== req.msg.id) {
          reject(new Error('ack_msg_id mismatch'))
          return
        }
        if (resp.msg.ack_from_id !== expectAckFrom) {
          reject(new Error('ack_from_id mismatch'))
          return
        }
        if (resp.msg.ack_to_id !== node) {
          reject(new Error('ack_to_id mismatch'))
          return
        }
        if (resp.msg.ack_version !== req.msg.version) {
          reject(new Error('ack_version mismatch'))
          return
        }
        console.log(`OK hops=${JSON.stringify(resp.hops)}`)
        resolve()
      } catch (err) {
        conn.destroy()
        reject(err)
      }
    })
    conn.on('error', reject)
  })
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2))
  const mode = args.mode
  const node = typeof args.node === 'string' ? args.node : 'ts'
  if (mode === 'server') {
    if (typeof args.listen !== 'string') throw new Error('missing --listen')
    const next = typeof args.next === 'string' ? args.next : undefined
    await runServer(args.listen, node, next, args.once === true)
    return
  }
  if (mode === 'client') {
    if (typeof args.addr !== 'string' || typeof args['expect-hops'] !== 'string' || typeof args['expect-ack-from'] !== 'string') {
      throw new Error('missing --addr, --expect-hops, or --expect-ack-from')
    }
    await runClient(args.addr, node, args['expect-hops'].split(','), args['expect-ack-from'])
    return
  }
  throw new Error(`unsupported mode: ${String(mode)}`)
}

void main().catch((err) => {
  console.error(`interop error: ${String(err)}`)
  process.exit(1)
})
