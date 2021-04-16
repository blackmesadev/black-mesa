package automod

import "regexp"

const ZALGOREGEX = `(?gu)[\p{Mn}\p{Me}]+`

// Return true if all is okay, return false if not.
func ZalgoCheck(m string) bool {
	r := regexp.MustCompile(ZALGOREGEX)

	match := r.MatchString(m)

	if match {
		return false
	}
	return true
}
