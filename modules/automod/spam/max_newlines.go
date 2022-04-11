package spam

import "strings"

func ProcessMaxNewlines(message string, limit int64) (bool, int64) {
	if limit == 0 {
		return true, 0
	}
	count := int64(strings.Count(message, "\n"))

	if count > limit {
		return false, count
	}

	return true, 0
}
