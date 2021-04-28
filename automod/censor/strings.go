package censor

import "strings"

func SubStringsCheck(m string, blacklist *[]string) (bool, string) {
	for _, substr := range *blacklist {
		if strings.Contains(m, substr) {
			return false, substr
		}
	}
	return true, ""
}

func StringsCheck(m string, blacklist *[]string) (bool, string) {
	words := strings.Split(m, " ")
	for _, str := range *blacklist {
		for _, word := range words {
			if word == str {
				return false, str
			}
		}
	}

	return true, ""
}