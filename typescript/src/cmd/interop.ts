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

async function runServerWithLimit(
  listen: string,
  node: string,
  next: string | undefined,
  once: boolean,
  maxRequests: number,
  ackMode: string,
  dropFirst: boolean,
): Promise<void> {
  const [host, portRaw] = listen.split(':')
  const port = Number(portRaw)
  let handled = 0
  const server = net.createServer((conn) => {
    void (async () => {
      try {
        const req = await readJSONLine(conn)
        if (dropFirst && handled === 0) {
          handled += 1
          conn.destroy()
          if (once || (maxRequests > 0 && handled >= maxRequests)) {
            server.close()
          }
          return
        }
        req.hops.push(node)
        const resp = next ? await forward(next, req) : {
          msg: {
            body: req.msg.id,
            from_id: node,
            id: `ack-${req.msg.id}`,
            to_ids: [req.msg.from_id],
            type_: 'Ack',
            version: req.msg.version,
            ack_msg_id: ackMode === 'missing_ack_msg_id' ? undefined : req.msg.id,
            ack_from_id: node,
            ack_to_id: req.msg.from_id,
            ack_version: ackMode === 'bad_ack_version' ? req.msg.version + 1 : req.msg.version,
          },
          hops: req.hops,
        }
        await writeJSONLine(conn, resp)
        conn.end()
        handled += 1
        if (once || (maxRequests > 0 && handled >= maxRequests)) {
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

async function runClient(
  addr: string,
  node: string,
  expectHops: string[],
  expectAckFrom: string,
  count: number,
  expectFailure: boolean,
  retries: number,
  retryDelayMS: number,
  timeoutMS: number,
): Promise<void> {
  const [host, portRaw] = addr.split(':')
  const port = Number(portRaw)
  let sawFailure = false
  for (let i = 0; i < count; i += 1) {
    let success = false
    for (let attempt = 0; attempt <= retries; attempt += 1) {
      // eslint-disable-next-line no-await-in-loop
      const valid = await new Promise<boolean>((resolve) => {
        const conn = net.createConnection({ host, port }, async () => {
          const timer = setTimeout(() => {
            conn.destroy()
            resolve(false)
          }, timeoutMS)
          try {
            const req: Envelope = {
              msg: {
                body: 'interop',
                from_id: node,
                id: `interop-msg-${i}`,
                to_ids: ['receiver'],
                type_: 'text',
                version: 1,
              },
              hops: [node],
            }
            await writeJSONLine(conn, req)
            const resp = await readJSONLine(conn)
            conn.end()
            clearTimeout(timer)
            resolve(
              JSON.stringify(resp.hops) === JSON.stringify(expectHops)
                && resp.msg.type_ === 'Ack'
                && resp.msg.ack_msg_id === req.msg.id
                && resp.msg.ack_from_id === expectAckFrom
                && resp.msg.ack_to_id === node
                && resp.msg.ack_version === req.msg.version,
            )
          } catch {
            clearTimeout(timer)
            conn.destroy()
            resolve(false)
          }
        })
        conn.on('error', () => resolve(false))
      })
      if (valid) {
        success = true
        if (expectFailure) throw new Error('expected failure but got valid ack')
        break
      }
      sawFailure = true
      if (attempt < retries) {
        // eslint-disable-next-line no-await-in-loop
        await new Promise((r) => setTimeout(r, retryDelayMS))
      }
    }
    if (!success && !expectFailure) {
      throw new Error(`interop validation failed for message ${i}`)
    }
  }
  if (expectFailure && !sawFailure) {
    throw new Error('expected at least one validation failure but saw none')
  }
  console.log(`OK count=${count} hops=${JSON.stringify(expectHops)}`)
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2))
  const mode = args.mode
  const node = typeof args.node === 'string' ? args.node : 'ts'
  if (mode === 'server') {
    if (typeof args.listen !== 'string') throw new Error('missing --listen')
    const next = typeof args.next === 'string' ? args.next : undefined
    const maxRequests = typeof args['max-requests'] === 'string' ? Number(args['max-requests']) : 0
    const ackMode = typeof args['ack-mode'] === 'string' ? args['ack-mode'] : 'normal'
    await runServerWithLimit(args.listen, node, next, args.once === true, maxRequests, ackMode, args['drop-first'] === true)
    return
  }
  if (mode === 'client') {
    if (typeof args.addr !== 'string' || typeof args['expect-hops'] !== 'string' || typeof args['expect-ack-from'] !== 'string') {
      throw new Error('missing --addr, --expect-hops, or --expect-ack-from')
    }
    const count = typeof args.count === 'string' ? Number(args.count) : 1
    const retries = typeof args.retries === 'string' ? Number(args.retries) : 0
    const retryDelayMS = typeof args['retry-delay-ms'] === 'string' ? Number(args['retry-delay-ms']) : 100
    const timeoutMS = typeof args['timeout-ms'] === 'string' ? Number(args['timeout-ms']) : 2000
    await runClient(
      args.addr,
      node,
      args['expect-hops'].split(','),
      args['expect-ack-from'],
      count,
      args['expect-failure'] === true,
      retries,
      retryDelayMS,
      timeoutMS,
    )
    return
  }
  throw new Error(`unsupported mode: ${String(mode)}`)
}

void main().catch((err) => {
  console.error(`interop error: ${String(err)}`)
  process.exit(1)
})
