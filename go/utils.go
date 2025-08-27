package bebopgen

import (
	"encoding/json"
	"fmt"
	"time"
)

// MsgUtils provides utility functions for Msg operations
type MsgUtils struct{}

// CreateWithTimestamp creates a new message with current timestamp
func (MsgUtils) CreateWithTimestamp(body, fromId string, toIds []string, msgType string) (*Msg, int64) {
	timestamp := time.Now().Unix()
	msg := &Msg{
		Body:   body,
		FromId: fromId,
		Id:     generateID(timestamp),
		ToIds:  toIds,
		Type:   msgType,
	}
	return msg, timestamp
}

// Validate checks if a message has all required fields
func (MsgUtils) Validate(msg *Msg) bool {
	return msg != nil &&
		msg.Body != "" &&
		msg.FromId != "" &&
		msg.Id != "" &&
		msg.Type != "" &&
		len(msg.ToIds) > 0
}

// GetSize returns the serialized size of the message
func (MsgUtils) GetSize(msg *Msg) int {
	if msg == nil {
		return 0
	}
	// Use the generated MarshalBebop method
	buf := make([]byte, msg.Size())
	size := msg.MarshalBebopTo(buf)
	return size
}

// ToJSON converts message to JSON string
func (MsgUtils) ToJSON(msg *Msg) (string, error) {
	bytes, err := json.Marshal(msg)
	return string(bytes), err
}

// FromJSON creates message from JSON string
func (MsgUtils) FromJSON(jsonStr string) (*Msg, error) {
	var msg Msg
	err := json.Unmarshal([]byte(jsonStr), &msg)
	return &msg, err
}

// generateID creates a unique ID based on timestamp
func generateID(timestamp int64) string {
	return fmt.Sprintf("msg_%d", timestamp)
}
