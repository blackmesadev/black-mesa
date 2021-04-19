package spam

import "strings"

func ProcessMaxNewlines(message string, limit int) bool {
	count := strings.Count(message, "\n")

	if count > limit {
		return false
	}

	return true
}