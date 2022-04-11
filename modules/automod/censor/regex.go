package censor

import (
	"regexp"
	"strings"
)

// Return true if all is okay, return false if not.
func RegexCheck(m string, regexString string) (string, bool) {

	regex, err := regexp.Compile(regexString)
	if err != nil {
		return "", true
	}

	matchSlice := regex.FindAllString(m, -1)
	if len(matchSlice) > 0 {
		matches := strings.Join(matchSlice, ", ")
		return matches, false
	}

	return "", true
}
