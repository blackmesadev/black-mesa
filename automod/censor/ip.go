package censor

import "regexp"

var ipRegex = regexp.MustCompile(`^(?:[0-255]{1,3}\.){3}[0-255]{1,3}$`)

// Return true if all is okay, return false if not.
func IPCheck(m string) bool {
	match := ipRegex.MatchString(m)

	return !match
}
