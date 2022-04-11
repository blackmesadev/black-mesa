package censor

import "testing"

func TestIP(t *testing.T) {
	if !IPCheck("127.0.0.1") {
		t.Error("Expected 127.0.0.1 to not be filtered")
	}

	if !IPCheck("489.37846.34789.2183") {
		t.Error("Expected 489.37846.34789.2183 to not be filtered")
	}

	if IPCheck("1.1.1.1") {
		t.Error("Expected 1.1.1.1 to be filtered")
	}

	if !IPCheck("1.893.1.1") {
		t.Error("Expected 1.893.1.1 to not be filtered")
	}

	if !IPCheck("2757235127.0.0.134567893467") {
		t.Error("Expected 2757235127.0.0.134567893467 to not be filtered")
	}
}
