package main_test

import (
	"path/filepath"
	"runtime"
	"testing"

	"github.com/jkassis/courier/pkg/conformance"
)

func TestConformanceFixture(t *testing.T) {
	_, here, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller failed")
	}
	fixturePath := filepath.Join(filepath.Dir(here), "..", "..", "conformance", "fixtures", "suite.v1.json")
	if err := conformance.RunFixture(fixturePath); err != nil {
		t.Fatalf("conformance fixture failed: %v", err)
	}
}
