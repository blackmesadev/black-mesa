package spam

import "strings"

func ProcessMaxNewlines(message string, limit int64) (bool, int64) {
	count := int64(strings.Count(message, "\n"))

	if count > limit {
		return false, count
	}

	return true, 0
}