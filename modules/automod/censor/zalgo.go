package censor

import "regexp"

var zalgoRegex = regexp.MustCompile(`[\p{Mn}\p{Me}]+`)

// Return true if all is okay, return false if not.
func ZalgoCheck(m string) bool {
	match := zalgoRegex.MatchString(m)

	return !match
}
