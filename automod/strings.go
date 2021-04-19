package automod

import "strings"

func StringsCheck(m string, blacklist *[]string) bool {
	for _, substr := range *blacklist {
		if strings.Contains(m, substr) {
			return false
		}
	}
	return true
}
