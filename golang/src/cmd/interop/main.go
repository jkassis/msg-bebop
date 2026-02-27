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

func runServer(listen string, node string, next string, once bool) error {
	ln, err := net.Listen("tcp", listen)
	if err != nil {
		return fmt.Errorf("listen %s: %w", listen, err)
	}
	defer ln.Close()

	for {
		conn, err := ln.Accept()
		if err != nil {
			return err
		}

		var in Envelope
		if err := readJSONLine(conn, &in); err != nil {
			_ = conn.Close()
			return err
		}
		in.Hops = append(in.Hops, node)

		var out Envelope
		if next != "" {
			upstream, err := net.Dial("tcp", next)
			if err != nil {
				_ = conn.Close()
				return fmt.Errorf("dial %s: %w", next, err)
			}
			if err := writeJSONLine(upstream, in); err != nil {
				_ = upstream.Close()
				_ = conn.Close()
				return err
			}
			if err := readJSONLine(upstream, &out); err != nil {
				_ = upstream.Close()
				_ = conn.Close()
				return err
			}
			_ = upstream.Close()
		} else {
			ackMsgID := in.Msg.ID
			ackFromID := node
			ackToID := in.Msg.FromID
			ackVersion := in.Msg.Version
			out = Envelope{
				Msg: Msg{
					Body:      in.Msg.ID,
					FromID:    node,
					ID:        "ack-" + in.Msg.ID,
					ToIDs:     []string{in.Msg.FromID},
					Type:      "Ack",
					Version:   in.Msg.Version,
					AckMsgID:  &ackMsgID,
					AckFromID: &ackFromID,
					AckToID:   &ackToID,
					AckVer:    &ackVersion,
				},
				Hops: in.Hops,
			}
		}

		if err := writeJSONLine(conn, out); err != nil {
			_ = conn.Close()
			return err
		}
		_ = conn.Close()

		if once {
			return nil
		}
	}
}

func runClient(addr string, node string, expectHops []string, expectAckFrom string) error {
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return err
	}
	defer conn.Close()

	msg := Msg{
		Body:    "interop",
		FromID:  node,
		ID:      "interop-msg",
		ToIDs:   []string{"receiver"},
		Type:    "text",
		Version: 1,
	}
	req := Envelope{
		Msg:  msg,
		Hops: []string{node},
	}
	if err := writeJSONLine(conn, req); err != nil {
		return err
	}
	var resp Envelope
	if err := readJSONLine(conn, &resp); err != nil {
		return err
	}
	if !slices.Equal(resp.Hops, expectHops) {
		return fmt.Errorf("unexpected hops: got %v want %v", resp.Hops, expectHops)
	}
	if resp.Msg.Type != "Ack" {
		return fmt.Errorf("expected Ack response, got %s", resp.Msg.Type)
	}
	if resp.Msg.AckMsgID == nil || *resp.Msg.AckMsgID != req.Msg.ID {
		return fmt.Errorf("ack_msg_id mismatch")
	}
	if resp.Msg.AckFromID == nil || *resp.Msg.AckFromID != expectAckFrom {
		return fmt.Errorf("ack_from_id mismatch")
	}
	if resp.Msg.AckToID == nil || *resp.Msg.AckToID != node {
		return fmt.Errorf("ack_to_id mismatch")
	}
	if resp.Msg.AckVer == nil || *resp.Msg.AckVer != req.Msg.Version {
		return fmt.Errorf("ack_version mismatch")
	}
	fmt.Printf("OK hops=%v\n", resp.Hops)
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
	once := flag.Bool("once", false, "serve a single request then exit")
	flag.Parse()

	var err error
	switch *mode {
	case "server":
		if *listen == "" {
			err = errors.New("missing -listen")
		} else {
			err = runServer(*listen, *node, *next, *once)
		}
	case "client":
		if *addr == "" || *expect == "" || *expectAckFrom == "" {
			err = errors.New("missing -addr, -expect-hops, or -expect-ack-from")
		} else {
			err = runClient(*addr, *node, strings.Split(*expect, ","), *expectAckFrom)
		}
	default:
		err = fmt.Errorf("unsupported mode %q", *mode)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "interop error: %v\n", err)
		os.Exit(1)
	}
}
