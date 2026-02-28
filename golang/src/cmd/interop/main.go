package main

import (
	"bufio"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"net"
	"os"
	"slices"
	"strings"
	"time"
)

type Msg struct {
	Body      string   `json:"body"`
	FromID    string   `json:"from_id"`
	ID        string   `json:"id"`
	ToIDs     []string `json:"to_ids"`
	Type      string   `json:"type_"`
	Version   uint16   `json:"version"`
	AckMsgID  *string  `json:"ack_msg_id,omitempty"`
	AckFromID *string  `json:"ack_from_id,omitempty"`
	AckToID   *string  `json:"ack_to_id,omitempty"`
	AckVer    *uint16  `json:"ack_version,omitempty"`
}

type Envelope struct {
	Msg  Msg      `json:"msg"`
	Hops []string `json:"hops"`
}

func writeJSONLine(conn net.Conn, v any) error {
	b, err := json.Marshal(v)
	if err != nil {
		return err
	}
	if _, err := conn.Write(append(b, '\n')); err != nil {
		return err
	}
	return nil
}

func readJSONLine(conn net.Conn, out any) error {
	line, err := bufio.NewReader(conn).ReadString('\n')
	if err != nil {
		return err
	}
	return json.Unmarshal([]byte(line), out)
}

func runServer(listen string, node string, next string, once bool, maxRequests int, ackMode string, dropFirst bool, delayMS int) error {
	ln, err := net.Listen("tcp", listen)
	if err != nil {
		return fmt.Errorf("listen %s: %w", listen, err)
	}
	defer ln.Close()
	fmt.Fprintf(os.Stderr, "INTEROP_READY %s\n", listen)

	handled := 0
	for {
		conn, err := ln.Accept()
		if err != nil {
			return err
		}
		requestIndex := handled
		handled++
		if delayMS > 0 {
			time.Sleep(time.Duration(delayMS) * time.Millisecond)
		}
		if dropFirst && requestIndex == 0 {
			_ = conn.Close()
			if once || (maxRequests > 0 && handled >= maxRequests) {
				return nil
			}
			continue
		}

		var in Envelope
		if err := readJSONLine(conn, &in); err != nil {
			_ = conn.Close()
			if once || (maxRequests > 0 && handled >= maxRequests) {
				return nil
			}
			continue
		}
		in.Hops = append(in.Hops, node)

		var out Envelope
		if next != "" {
			upstream, err := net.Dial("tcp", next)
			if err != nil {
				_ = conn.Close()
				if once || (maxRequests > 0 && handled >= maxRequests) {
					return nil
				}
				continue
			}
			if err := writeJSONLine(upstream, in); err != nil {
				_ = upstream.Close()
				_ = conn.Close()
				if once || (maxRequests > 0 && handled >= maxRequests) {
					return nil
				}
				continue
			}
			if err := readJSONLine(upstream, &out); err != nil {
				_ = upstream.Close()
				_ = conn.Close()
				if once || (maxRequests > 0 && handled >= maxRequests) {
					return nil
				}
				continue
			}
			_ = upstream.Close()
		} else {
			ackMsgID := in.Msg.ID
			ackFromID := node
			ackToID := in.Msg.FromID
			ackVersion := in.Msg.Version
			var ackMsgIDPtr *string
			if ackMode != "missing_ack_msg_id" {
				ackMsgIDPtr = &ackMsgID
			}
			if ackMode == "bad_ack_version" {
				ackVersion = in.Msg.Version + 1
			}
			out = Envelope{
				Msg: Msg{
					Body:      in.Msg.ID,
					FromID:    node,
					ID:        "ack-" + in.Msg.ID,
					ToIDs:     []string{in.Msg.FromID},
					Type:      "Ack",
					Version:   in.Msg.Version,
					AckMsgID:  ackMsgIDPtr,
					AckFromID: &ackFromID,
					AckToID:   &ackToID,
					AckVer:    &ackVersion,
				},
				Hops: in.Hops,
			}
		}

		if err := writeJSONLine(conn, out); err != nil {
			_ = conn.Close()
			if once || (maxRequests > 0 && handled >= maxRequests) {
				return nil
			}
			continue
		}
		_ = conn.Close()

		if once || (maxRequests > 0 && handled >= maxRequests) {
			return nil
		}
	}
}

func runClient(addr string, node string, expectHops []string, expectAckFrom string, count int, expectFailure bool, retries int, retryDelayMS int, timeoutMS int) error {
	sawFailure := false
	for i := 0; i < count; i++ {
		success := false
		for attempt := 0; attempt <= retries; attempt++ {
			valid := false
			conn, err := net.Dial("tcp", addr)
			if err == nil {
				_ = conn.SetDeadline(time.Now().Add(time.Duration(timeoutMS) * time.Millisecond))

				msg := Msg{
					Body:    "interop",
					FromID:  node,
					ID:      fmt.Sprintf("interop-msg-%d", i),
					ToIDs:   []string{"receiver"},
					Type:    "text",
					Version: 1,
				}
				req := Envelope{
					Msg:  msg,
					Hops: []string{node},
				}
				if err := writeJSONLine(conn, req); err == nil {
					var resp Envelope
					if err := readJSONLine(conn, &resp); err == nil {
						valid = slices.Equal(resp.Hops, expectHops) &&
							resp.Msg.Type == "Ack" &&
							resp.Msg.AckMsgID != nil && *resp.Msg.AckMsgID == req.Msg.ID &&
							resp.Msg.AckFromID != nil && *resp.Msg.AckFromID == expectAckFrom &&
							resp.Msg.AckToID != nil && *resp.Msg.AckToID == node &&
							resp.Msg.AckVer != nil && *resp.Msg.AckVer == req.Msg.Version
					}
				}
				_ = conn.Close()
			}
			if valid {
				success = true
				if expectFailure {
					return fmt.Errorf("expected failure but got valid ack")
				}
				break
			}
			sawFailure = true
			if attempt < retries {
				time.Sleep(time.Duration(retryDelayMS) * time.Millisecond)
			}
		}
		if !success && !expectFailure {
			return fmt.Errorf("interop validation failed for message %d", i)
		}
	}
	if expectFailure && !sawFailure {
		return fmt.Errorf("expected at least one validation failure but saw none")
	}
	fmt.Printf("OK count=%d hops=%v\n", count, expectHops)
	return nil
}

func main() {
	mode := flag.String("mode", "", "server|client")
	node := flag.String("node", "go", "node name")
	listen := flag.String("listen", "", "listen addr for server")
	next := flag.String("next", "", "next addr for server")
	addr := flag.String("addr", "", "addr for client")
	expect := flag.String("expect-hops", "", "comma-separated expected hops for client")
	expectAckFrom := flag.String("expect-ack-from", "", "expected terminal ack sender for client")
	count := flag.Int("count", 1, "number of client messages")
	maxRequests := flag.Int("max-requests", 0, "max server requests before exit")
	ackMode := flag.String("ack-mode", "normal", "normal|missing_ack_msg_id|bad_ack_version")
	expectFailure := flag.Bool("expect-failure", false, "client expects invalid ack validation")
	retries := flag.Int("retries", 0, "client retries per message")
	retryDelayMS := flag.Int("retry-delay-ms", 100, "client retry delay in milliseconds")
	timeoutMS := flag.Int("timeout-ms", 2000, "client timeout in milliseconds")
	dropFirst := flag.Bool("drop-first", false, "server drops first request connection")
	delayMS := flag.Int("delay-ms", 0, "server delay before handling each request in milliseconds")
	once := flag.Bool("once", false, "serve a single request then exit")
	flag.Parse()

	var err error
	switch *mode {
	case "server":
		if *listen == "" {
			err = errors.New("missing -listen")
		} else {
			err = runServer(*listen, *node, *next, *once, *maxRequests, *ackMode, *dropFirst, *delayMS)
		}
	case "client":
		if *addr == "" || *expect == "" || *expectAckFrom == "" {
			err = errors.New("missing -addr, -expect-hops, or -expect-ack-from")
		} else {
			err = runClient(*addr, *node, strings.Split(*expect, ","), *expectAckFrom, *count, *expectFailure, *retries, *retryDelayMS, *timeoutMS)
		}
	default:
		err = fmt.Errorf("unsupported mode %q", *mode)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "interop error: %v\n", err)
		os.Exit(1)
	}
}
