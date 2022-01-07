package censor

import (
	"regexp"
	"strings"
)

var ipRegex = regexp.MustCompile(`(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}`)

// Return true if all is okay, return false if not.
func IPCheck(m string) bool {
	match := ipRegex.FindAllString(m, -1)

	// Check if ips exist
	if len(match) == 0 {
		return true
	}

	for _, m := range match {
		// Exclude a few known known public ips - there are more, will come back to this
		if strings.HasPrefix(m, "192.") || strings.HasPrefix(m, "127.") || strings.HasPrefix(m, "0.") {
			continue
		}
		return false
	}

	return true
}
