package conformance

import (
	"encoding/json"
	"fmt"
	"os"
	"slices"
)

type RawMsg struct {
	Body      string   `json:"body"`
	FromID    string   `json:"from_id"`
	ID        string   `json:"id"`
	ToIDs     []string `json:"to_ids"`
	Type      string   `json:"type_"`
	Version   *uint16  `json:"version,omitempty"`
	AckMsgID  *string  `json:"ack_msg_id,omitempty"`
	AckFromID *string  `json:"ack_from_id,omitempty"`
	AckToID   *string  `json:"ack_to_id,omitempty"`
	AckVer    *uint16  `json:"ack_version,omitempty"`
}

type Msg struct {
	Body      string
	FromID    string
	ID        string
	ToIDs     []string
	Type      string
	Version   uint16
	AckMsgID  *string
	AckFromID *string
	AckToID   *string
	AckVer    *uint16
}

type Suite struct {
	MessageDecode []MessageDecodeCase `json:"message_decode"`
	TxValidate    []TxValidateCase    `json:"tx_validate"`
	AckApply      []AckApplyCase      `json:"ack_apply"`
}

type MessageDecodeCase struct {
	Name     string          `json:"name"`
	Input    json.RawMessage `json:"input"`
	Expected struct {
		Version uint16 `json:"version"`
	} `json:"expected"`
}

type TxValidateCase struct {
	Name              string `json:"name"`
	Msg               RawMsg `json:"msg"`
	ExpectedErrorCode string `json:"expected_error_code"`
}

type AckApplyCase struct {
	Name              string       `json:"name"`
	CourierID         string       `json:"courier_id"`
	TxMsg             RawMsg       `json:"tx_msg"`
	AckMsg            RawMsg       `json:"ack_msg"`
	Expected          *AckExpected `json:"expected,omitempty"`
	ExpectedErrorCode *string      `json:"expected_error_code,omitempty"`
}

type AckExpected struct {
	Deleted        bool     `json:"deleted"`
	RemainingToIDs []string `json:"remaining_to_ids"`
}

type AckApplyResult struct {
	Deleted        bool
	RemainingToIDs []string
}

func decodeMsg(raw RawMsg) Msg {
	version := uint16(1)
	if raw.Version != nil {
		version = *raw.Version
	}
	return Msg{
		Body:      raw.Body,
		FromID:    raw.FromID,
		ID:        raw.ID,
		ToIDs:     raw.ToIDs,
		Type:      raw.Type,
		Version:   version,
		AckMsgID:  raw.AckMsgID,
		AckFromID: raw.AckFromID,
		AckToID:   raw.AckToID,
		AckVer:    raw.AckVer,
	}
}

func validateTxMsg(msg Msg) error {
	if msg.Version == 0 {
		return fmt.Errorf("invalid_msg_version")
	}
	hasAckFields := msg.AckMsgID != nil || msg.AckFromID != nil || msg.AckToID != nil || msg.AckVer != nil
	if msg.Type != "Ack" && hasAckFields {
		return fmt.Errorf("non_ack_has_ack_fields")
	}
	return nil
}

func applyAck(courierID string, txMsg Msg, ackMsg Msg) (AckApplyResult, error) {
	if ackMsg.AckMsgID == nil {
		return AckApplyResult{}, fmt.Errorf("missing_ack_msg_id")
	}
	if ackMsg.AckFromID == nil {
		return AckApplyResult{}, fmt.Errorf("missing_ack_from_id")
	}
	if ackMsg.AckToID == nil {
		return AckApplyResult{}, fmt.Errorf("missing_ack_to_id")
	}
	if ackMsg.AckVer == nil {
		return AckApplyResult{}, fmt.Errorf("missing_ack_version")
	}
	if ackMsg.FromID != *ackMsg.AckFromID {
		return AckApplyResult{}, fmt.Errorf("ack_from_mismatch")
	}
	if !slices.Contains(ackMsg.ToIDs, *ackMsg.AckToID) {
		return AckApplyResult{}, fmt.Errorf("ack_to_missing_in_envelope")
	}
	if *ackMsg.AckToID != courierID {
		return AckApplyResult{}, fmt.Errorf("ack_to_wrong_courier")
	}
	if ackMsg.Version != *ackMsg.AckVer {
		return AckApplyResult{}, fmt.Errorf("ack_envelope_version_mismatch")
	}
	if *ackMsg.AckMsgID != txMsg.ID {
		return AckApplyResult{}, fmt.Errorf("ack_msg_id_mismatch")
	}
	if *ackMsg.AckVer != txMsg.Version {
		return AckApplyResult{}, fmt.Errorf("ack_version_mismatch")
	}

	remaining := make([]string, 0, len(txMsg.ToIDs))
	for _, toID := range txMsg.ToIDs {
		if toID != *ackMsg.AckFromID {
			remaining = append(remaining, toID)
		}
	}
	if len(remaining) == len(txMsg.ToIDs) {
		return AckApplyResult{}, fmt.Errorf("ack_from_not_pending")
	}
	slices.Sort(remaining)
	return AckApplyResult{Deleted: len(remaining) == 0, RemainingToIDs: remaining}, nil
}

func RunFixture(path string) error {
	raw, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("read fixture: %w", err)
	}

	var suite Suite
	if err := json.Unmarshal(raw, &suite); err != nil {
		return fmt.Errorf("parse fixture: %w", err)
	}

	for _, c := range suite.MessageDecode {
		var rawMsg RawMsg
		if err := json.Unmarshal(c.Input, &rawMsg); err != nil {
			return fmt.Errorf("case %s decode input: %w", c.Name, err)
		}
		msg := decodeMsg(rawMsg)
		if msg.Version != c.Expected.Version {
			return fmt.Errorf("case %s expected version %d got %d", c.Name, c.Expected.Version, msg.Version)
		}
	}

	for _, c := range suite.TxValidate {
		err := validateTxMsg(decodeMsg(c.Msg))
		if err == nil {
			return fmt.Errorf("case %s expected tx validation error %s", c.Name, c.ExpectedErrorCode)
		}
		if err.Error() != c.ExpectedErrorCode {
			return fmt.Errorf("case %s expected error %s got %s", c.Name, c.ExpectedErrorCode, err.Error())
		}
	}

	for _, c := range suite.AckApply {
		result, err := applyAck(c.CourierID, decodeMsg(c.TxMsg), decodeMsg(c.AckMsg))
		if c.ExpectedErrorCode != nil {
			if err == nil {
				return fmt.Errorf("case %s expected ack error %s", c.Name, *c.ExpectedErrorCode)
			}
			if err.Error() != *c.ExpectedErrorCode {
				return fmt.Errorf("case %s expected ack error %s got %s", c.Name, *c.ExpectedErrorCode, err.Error())
			}
			continue
		}
		if err != nil {
			return fmt.Errorf("case %s unexpected ack error: %w", c.Name, err)
		}
		if c.Expected == nil {
			return fmt.Errorf("case %s missing expected success payload", c.Name)
		}
		got := slices.Clone(result.RemainingToIDs)
		want := slices.Clone(c.Expected.RemainingToIDs)
		slices.Sort(got)
		slices.Sort(want)
		if result.Deleted != c.Expected.Deleted {
			return fmt.Errorf("case %s expected deleted=%v got %v", c.Name, c.Expected.Deleted, result.Deleted)
		}
		if !slices.Equal(got, want) {
			return fmt.Errorf("case %s expected remaining_to_ids=%v got %v", c.Name, want, got)
		}
	}

	return nil
}
