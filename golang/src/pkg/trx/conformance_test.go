package trx

import (
	"encoding/base64"
	"encoding/json"
	"os"
	"testing"
)

type fixtureSuite struct {
	MessageDecode []fixtureMessageDecodeCase `json:"message_decode"`
}

type fixtureMessageDecodeCase struct {
	Name     string          `json:"name"`
	Input    json.RawMessage `json:"input"`
	Expected struct {
		Version    uint16 `json:"version"`
		BodyBase64 string `json:"body_base64"`
	} `json:"expected"`
}

func TestTrxConformanceMessageDecodeCases(t *testing.T) {
	raw, err := os.ReadFile("../../../../conformance/fixtures/trx_suite.v1.json")
	if err != nil {
		t.Fatalf("read fixture: %v", err)
	}

	var suite fixtureSuite
	if err := json.Unmarshal(raw, &suite); err != nil {
		t.Fatalf("parse fixture: %v", err)
	}

	for _, tc := range suite.MessageDecode {
		var msg Msg
		if err := json.Unmarshal(tc.Input, &msg); err != nil {
			t.Fatalf("case %s decode: %v", tc.Name, err)
		}
		msg.Normalize()

		if msg.Version != tc.Expected.Version {
			t.Fatalf("case %s version=%d want %d", tc.Name, msg.Version, tc.Expected.Version)
		}
		if got := base64.StdEncoding.EncodeToString(msg.Body); got != tc.Expected.BodyBase64 {
			t.Fatalf("case %s body=%s want %s", tc.Name, got, tc.Expected.BodyBase64)
		}
	}
}
