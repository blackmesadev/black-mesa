package automod

import "regexp"

var regex = regexp.MustCompile(`[\p{Mn}\p{Me}]+`)

// Return true if all is okay, return false if not.
func ZalgoCheck(m string) bool {
	match := regex.MatchString(m)

	return !match
}
