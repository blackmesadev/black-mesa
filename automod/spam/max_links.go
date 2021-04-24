package spam

import "regexp"

var domainsRegex = regexp.MustCompile(`https?://[^\s]+`)

func ProcessMaxLinks(message string, limit int) (bool, int) {
	count := len(domainsRegex.FindAllStringIndex(message, -1))
	return count <= limit, count
}