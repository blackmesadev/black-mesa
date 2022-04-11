package automod

import "regexp"

var nonStdSpaceRegex = regexp.MustCompile(`[\x{2000}-\x{200F}]+`)

func replaceNonStandardSpace(m string) string {
	return nonStdSpaceRegex.ReplaceAllString(m, " ")
}
