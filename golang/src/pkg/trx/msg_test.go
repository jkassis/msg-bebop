package trx

import (
	"encoding/json"
	"testing"
)

func TestMsgJSONRoundTripUsesBase64Body(t *testing.T) {
	msg := NewMsg("m1", "sender", []string{"receiver"}, "event", []byte{0, 1, 2, 255})

	raw, err := json.Marshal(msg)
	if err != nil {
		t.Fatalf("marshal msg: %v", err)
	}
	if got, want := string(raw), `{"id":"m1","from_id":"sender","to_ids":["receiver"],"type_":"event","version":1,"body":"AAEC/w=="}`; got != want {
		t.Fatalf("marshal msg = %s; want %s", got, want)
	}

	var decoded Msg
	if err := json.Unmarshal(raw, &decoded); err != nil {
		t.Fatalf("unmarshal msg: %v", err)
	}
	decoded.Normalize()

	if decoded.Version != 1 {
		t.Fatalf("decoded version = %d; want 1", decoded.Version)
	}
	if string(decoded.Body) != string(msg.Body) {
		t.Fatalf("decoded body = %v; want %v", decoded.Body, msg.Body)
	}
}

func TestMsgDefaultsVersionToOneAfterNormalize(t *testing.T) {
	raw := []byte(`{"id":"m1","from_id":"sender","to_ids":["receiver"],"type_":"event","body":"aGk="}`)

	var decoded Msg
	if err := json.Unmarshal(raw, &decoded); err != nil {
		t.Fatalf("unmarshal msg: %v", err)
	}
	decoded.Normalize()

	if decoded.Version != 1 {
		t.Fatalf("decoded version = %d; want 1", decoded.Version)
	}
	if string(decoded.Body) != "hi" {
		t.Fatalf("decoded body = %q; want hi", string(decoded.Body))
	}
}
