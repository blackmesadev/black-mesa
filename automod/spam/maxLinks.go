package spam

import "regexp"

var domainsRegex = regexp.MustCompile(`https?://[^\s]+`)

func ProcessMaxLinks(message string, limit int) bool {
	return len(domainsRegex.FindAllStringIndex(message, -1)) <= limit
}