use courier::Msg;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Envelope {
    msg: Msg,
    hops: Vec<String>,
}

fn parse_arg(args: &[String], key: &str) -> Option<String> {
    let flag = format!("--{key}");
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn has_flag(args: &[String], key: &str) -> bool {
    let flag = format!("--{key}");
    args.iter().any(|arg| arg == &flag)
}

fn parse_usize_arg(args: &[String], key: &str, default: usize) -> Result<usize, String> {
    match parse_arg(args, key) {
        Some(v) => v
            .parse::<usize>()
            .map_err(|e| format!("invalid --{key} value {v}: {e}")),
        None => Ok(default),
    }
}

fn parse_u64_arg(args: &[String], key: &str, default: u64) -> Result<u64, String> {
    match parse_arg(args, key) {
        Some(v) => v
            .parse::<u64>()
            .map_err(|e| format!("invalid --{key} value {v}: {e}")),
        None => Ok(default),
    }
}

fn read_line_json<T: for<'de> Deserialize<'de>>(stream: &TcpStream) -> Result<T, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let n = reader
        .read_line(&mut line)
        .map_err(|e| format!("read_line failed: {e}"))?;
    if n == 0 {
        return Err("connection closed before payload".to_string());
    }
    serde_json::from_str::<T>(&line).map_err(|e| format!("invalid json payload: {e}"))
}

fn write_line_json<T: Serialize>(stream: &mut TcpStream, v: &T) -> Result<(), String> {
    let payload = serde_json::to_string(v).map_err(|e| format!("serialize failed: {e}"))?;
    stream
        .write_all(payload.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|e| format!("write failed: {e}"))
}

fn run_server(
    listen: &str,
    node: &str,
    next: Option<&str>,
    once: bool,
    max_requests: Option<usize>,
    ack_mode: &str,
    drop_first: bool,
) -> Result<(), String> {
    let listener = TcpListener::bind(listen).map_err(|e| format!("bind {listen} failed: {e}"))?;
    let mut handled = 0usize;
    loop {
        let (mut conn, _addr) = listener
            .accept()
            .map_err(|e| format!("accept failed: {e}"))?;
        if drop_first && handled == 0 {
            handled += 1;
            continue;
        }
        let mut env: Envelope = read_line_json(&conn)?;
        env.hops.push(node.to_string());

        let response = if let Some(next_addr) = next {
            let mut upstream = TcpStream::connect(next_addr)
                .map_err(|e| format!("dial {next_addr} failed: {e}"))?;
            write_line_json(&mut upstream, &env)?;
            read_line_json::<Envelope>(&upstream)?
        } else {
            let req = env;
            Envelope {
                msg: Msg {
                    id: format!("ack-{}", req.msg.id),
                    from_id: node.to_string(),
                    to_ids: vec![req.msg.from_id.clone()],
                    type_: "Ack".to_string(),
                    body: req.msg.id.clone(),
                    version: req.msg.version,
                    ack_msg_id: if ack_mode == "missing_ack_msg_id" {
                        None
                    } else {
                        Some(req.msg.id.clone())
                    },
                    ack_from_id: Some(node.to_string()),
                    ack_to_id: Some(req.msg.from_id.clone()),
                    ack_version: Some(if ack_mode == "bad_ack_version" {
                        req.msg.version.saturating_add(1)
                    } else {
                        req.msg.version
                    }),
                },
                hops: req.hops,
            }
        };

        write_line_json(&mut conn, &response)?;
        handled += 1;
        if once || max_requests.is_some_and(|limit| handled >= limit) {
            return Ok(());
        }
    }
}

fn run_client(
    addr: &str,
    node: &str,
    expect_hops: &[String],
    expect_ack_from: &str,
    count: usize,
    expect_failure: bool,
    retries: usize,
    retry_delay_ms: u64,
    timeout_ms: u64,
) -> Result<(), String> {
    let mut saw_failure = false;
    for i in 0..count {
        let mut success = false;
        for attempt in 0..=retries {
            let mut stream =
                TcpStream::connect(addr).map_err(|e| format!("dial {addr} failed: {e}"))?;
            stream
                .set_read_timeout(Some(Duration::from_millis(timeout_ms)))
                .map_err(|e| format!("set_read_timeout failed: {e}"))?;
            stream
                .set_write_timeout(Some(Duration::from_millis(timeout_ms)))
                .map_err(|e| format!("set_write_timeout failed: {e}"))?;
            let msg_id = format!("interop-msg-{i}");
            let env = Envelope {
                msg: Msg {
                    body: "interop".to_string(),
                    from_id: node.to_string(),
                    id: msg_id,
                    to_ids: vec!["receiver".to_string()],
                    type_: "text".to_string(),
                    version: 1,
                    ack_msg_id: None,
                    ack_from_id: None,
                    ack_to_id: None,
                    ack_version: None,
                },
                hops: vec![node.to_string()],
            };
            let valid = match write_line_json(&mut stream, &env)
                .and_then(|_| read_line_json::<Envelope>(&stream))
                .map(|response| {
                    response.hops == expect_hops
                        && response.msg.type_ == "Ack"
                        && response.msg.ack_msg_id.as_deref() == Some(env.msg.id.as_str())
                        && response.msg.ack_from_id.as_deref() == Some(expect_ack_from)
                        && response.msg.ack_to_id.as_deref() == Some(node)
                        && response.msg.ack_version == Some(env.msg.version)
                }) {
                Ok(v) => v,
                Err(_) => false,
            };
            if valid {
                success = true;
                if expect_failure {
                    return Err("expected failure but received valid ack".to_string());
                }
                break;
            }
            saw_failure = true;
            if attempt < retries {
                std::thread::sleep(Duration::from_millis(retry_delay_ms));
            }
        }
        if !success && !expect_failure {
            return Err(format!("interop validation failed for message {i}"));
        }
    }
    if expect_failure && !saw_failure {
        return Err("expected at least one validation failure but saw none".to_string());
    }
    println!("OK count={count} hops={:?}", expect_hops);
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_arg(&args, "mode").ok_or_else(|| "missing --mode".to_string())?;
    let node = parse_arg(&args, "node").unwrap_or_else(|| "rust".to_string());

    match mode.as_str() {
        "server" => {
            let listen =
                parse_arg(&args, "listen").ok_or_else(|| "missing --listen".to_string())?;
            let next = parse_arg(&args, "next");
            let max_requests = parse_arg(&args, "max-requests")
                .map(|v| {
                    v.parse::<usize>()
                        .map_err(|e| format!("invalid --max-requests value {v}: {e}"))
                })
                .transpose()?;
            let ack_mode = parse_arg(&args, "ack-mode").unwrap_or_else(|| "normal".to_string());
            let drop_first = has_flag(&args, "drop-first");
            run_server(
                &listen,
                &node,
                next.as_deref(),
                has_flag(&args, "once"),
                max_requests,
                &ack_mode,
                drop_first,
            )
        }
        "client" => {
            let addr = parse_arg(&args, "addr").ok_or_else(|| "missing --addr".to_string())?;
            let expect = parse_arg(&args, "expect-hops")
                .ok_or_else(|| "missing --expect-hops".to_string())?;
            let expect_ack_from = parse_arg(&args, "expect-ack-from")
                .ok_or_else(|| "missing --expect-ack-from".to_string())?;
            let expect_hops = expect.split(',').map(|s| s.to_string()).collect::<Vec<_>>();
            let count = parse_usize_arg(&args, "count", 1)?;
            let retries = parse_usize_arg(&args, "retries", 0)?;
            let retry_delay_ms = parse_u64_arg(&args, "retry-delay-ms", 100)?;
            let timeout_ms = parse_u64_arg(&args, "timeout-ms", 2000)?;
            run_client(
                &addr,
                &node,
                &expect_hops,
                &expect_ack_from,
                count,
                has_flag(&args, "expect-failure"),
                retries,
                retry_delay_ms,
                timeout_ms,
            )
        }
        _ => Err(format!("unsupported mode: {mode}")),
    }
}
