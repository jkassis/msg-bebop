// golang/tests/math_test.go
package main_test

import (
	"testing"

	"github.com/jkassis/courier/pkg" // Public API access only
)

func TestAddIntegration(t *testing.T) {
	got := pkg.Add(10, 5)
	want := 15
	if got != want {
		t.Errorf("expected %d, got %d", want, got)
	}
}
