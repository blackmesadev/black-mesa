package censor

import "strings"

func StringsCheck(m string, blacklist *[]string) (bool, string) {
	for _, substr := range *blacklist {
		if strings.Contains(m, substr) {
			return false, substr
		}
	}
	return true, ""
}
