package bebopgen

import (
	"reflect"
	"testing"
	"time"
)

func TestMsgSerialization(t *testing.T) {
	original := &Msg{
		Body:   "Hello from Go!",
		FromId: "go_test",
		Id:     "test_001",
		ToIds:  []string{"user1", "user2"},
		Type:   "test",
	}

	// Serialize using generated API
	buf := make([]byte, original.Size())
	size := original.MarshalBebopTo(buf)
	if size == 0 {
		t.Fatal("Serialization failed - empty bytes")
	}
	t.Logf("Serialized size: %d bytes", size)

	// Deserialize using generated API
	decoded := &Msg{}
	if err := decoded.UnmarshalBebop(buf[:size]); err != nil {
		t.Fatalf("Deserialization failed: %v", err)
	}

	// Verify fields
	if decoded.Body != original.Body {
		t.Errorf("Body mismatch: got %s, want %s", decoded.Body, original.Body)
	}
	if decoded.FromId != original.FromId {
		t.Errorf("FromId mismatch: got %s, want %s", decoded.FromId, original.FromId)
	}
	if decoded.Id != original.Id {
		t.Errorf("Id mismatch: got %s, want %s", decoded.Id, original.Id)
	}
	if !reflect.DeepEqual(decoded.ToIds, original.ToIds) {
		t.Errorf("ToIds mismatch: got %v, want %v", decoded.ToIds, original.ToIds)
	}
	if decoded.Type != original.Type {
		t.Errorf("Type mismatch: got %s, want %s", decoded.Type, original.Type)
	}

	t.Log("✅ Go serialization test passed!")
}

func TestMsgUtils(t *testing.T) {
	utils := MsgUtils{}

	// Test CreateWithTimestamp
	msg, timestamp := utils.CreateWithTimestamp(
		"Test message",
		"sender",
		[]string{"recipient1", "recipient2"},
		"utility_test",
	)

	if msg == nil {
		t.Fatal("CreateWithTimestamp returned nil message")
	}

	// Verify timestamp is reasonable
	if timestamp <= 0 {
		t.Error("CreateWithTimestamp should return valid timestamp")
	}

	// Test Validate
	if !utils.Validate(msg) {
		t.Error("Validate should return true for valid message")
	}

	// Test with invalid message
	invalidMsg := &Msg{}
	if utils.Validate(invalidMsg) {
		t.Error("Validate should return false for invalid message")
	}

	// Test GetSize
	size := utils.GetSize(msg)
	if size == 0 {
		t.Error("GetSize should return non-zero for valid message")
	}
	t.Logf("Message size: %d bytes", size)

	// Test JSON conversion
	jsonStr, err := utils.ToJSON(msg)
	if err != nil {
		t.Fatalf("ToJSON failed: %v", err)
	}

	reconstructed, err := utils.FromJSON(jsonStr)
	if err != nil {
		t.Fatalf("FromJSON failed: %v", err)
	}

	if reconstructed.Body != msg.Body {
		t.Errorf("JSON round-trip failed for Body: got %s, want %s",
			reconstructed.Body, msg.Body)
	}

	t.Log("✅ Go utilities test passed!")
}

func TestMsgPerformance(t *testing.T) {
	msg := &Msg{
		Body:   "Performance test message with some content",
		FromId: "perf_test",
		Id:     "perf_001",
		ToIds:  []string{"user1", "user2", "user3", "user4"},
		Type:   "performance",
	}

	// Benchmark serialization
	iterations := 10000
	start := time.Now()

	buf := make([]byte, msg.Size())
	for i := 0; i < iterations; i++ {
		size := msg.MarshalBebopTo(buf)
		decoded := &Msg{}
		if err := decoded.UnmarshalBebop(buf[:size]); err != nil {
			t.Fatalf("Performance test failed at iteration %d: %v", i, err)
		}
	}

	elapsed := time.Since(start)
	opsPerSec := float64(iterations) / elapsed.Seconds()

	t.Logf("🏃 Performance: %.0f ops/sec (%d iterations in %v)",
		opsPerSec, iterations, elapsed)
}
