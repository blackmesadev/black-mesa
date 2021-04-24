package spam

import "strings"

func ProcessMaxNewlines(message string, limit int) (bool, int) {
	count := strings.Count(message, "\n")

	if count > limit {
		return false, count
	}

	return true, 0
}