package spam

import "regexp"

var domainsRegex = regexp.MustCompile(`https?://[^\s]+`)

func ProcessMaxLinks(message string, limit int64) (bool, int64) {
	if limit == 0 {
		return true, 0
	}
	count := int64(len(domainsRegex.FindAllStringIndex(message, -1)))
	return count <= limit, count
}
