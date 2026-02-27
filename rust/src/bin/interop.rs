use courier::Msg;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

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

fn run_server(listen: &str, node: &str, next: Option<&str>, once: bool) -> Result<(), String> {
    let listener = TcpListener::bind(listen).map_err(|e| format!("bind {listen} failed: {e}"))?;

    loop {
        let (mut conn, _addr) = listener
            .accept()
            .map_err(|e| format!("accept failed: {e}"))?;
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
                    ack_msg_id: Some(req.msg.id.clone()),
                    ack_from_id: Some(node.to_string()),
                    ack_to_id: Some(req.msg.from_id.clone()),
                    ack_version: Some(req.msg.version),
                },
                hops: req.hops,
            }
        };

        write_line_json(&mut conn, &response)?;
        if once {
            return Ok(());
        }
    }
}

fn run_client(
    addr: &str,
    node: &str,
    expect_hops: &[String],
    expect_ack_from: &str,
) -> Result<(), String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| format!("dial {addr} failed: {e}"))?;
    let env = Envelope {
        msg: Msg {
            body: "interop".to_string(),
            from_id: node.to_string(),
            id: "interop-msg".to_string(),
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
    write_line_json(&mut stream, &env)?;
    let response: Envelope = read_line_json(&stream)?;
    if response.hops != expect_hops {
        return Err(format!(
            "unexpected hops: got {:?}, want {:?}",
            response.hops, expect_hops
        ));
    }
    if response.msg.type_ != "Ack" {
        return Err(format!("expected Ack response, got {}", response.msg.type_));
    }
    if response.msg.ack_msg_id.as_deref() != Some(env.msg.id.as_str()) {
        return Err("ack_msg_id mismatch".to_string());
    }
    if response.msg.ack_from_id.as_deref() != Some(expect_ack_from) {
        return Err("ack_from_id mismatch".to_string());
    }
    if response.msg.ack_to_id.as_deref() != Some(node) {
        return Err("ack_to_id mismatch".to_string());
    }
    if response.msg.ack_version != Some(env.msg.version) {
        return Err("ack_version mismatch".to_string());
    }
    println!("OK hops={:?}", response.hops);
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
            run_server(&listen, &node, next.as_deref(), has_flag(&args, "once"))
        }
        "client" => {
            let addr = parse_arg(&args, "addr").ok_or_else(|| "missing --addr".to_string())?;
            let expect = parse_arg(&args, "expect-hops")
                .ok_or_else(|| "missing --expect-hops".to_string())?;
            let expect_ack_from = parse_arg(&args, "expect-ack-from")
                .ok_or_else(|| "missing --expect-ack-from".to_string())?;
            let expect_hops = expect.split(',').map(|s| s.to_string()).collect::<Vec<_>>();
            run_client(&addr, &node, &expect_hops, &expect_ack_from)
        }
        _ => Err(format!("unsupported mode: {mode}")),
    }
}
