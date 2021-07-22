package censor

import (
	"fmt"
	"regexp"
)

var domainsRegex = regexp.MustCompile(`(?:https?:\/\/)?(?:[^@\/\n]+@)?(?:www\.)?([^:\/\n]+)`)

func DomainsWhitelistCheck(m string, whitelist *[]string) (bool, string) {
	ok := false

	domains := domainsRegex.FindAllString(m, -1)

	if len(domains) == 0 {
		return true, ""
	}

	fmt.Println(domains)

	for _, domain := range domains {
		for _, whitelistedDomain := range *whitelist {
			if domain == whitelistedDomain {
				ok = true
			} else {
				return false, domain
			}
		}
	}

	return ok, ""
}

func DomainsBlacklistCheck(m string, blacklist *[]string) (bool, string) {
	ok := true
	domains := domainsRegex.FindAllString(m, -1)

	if len(domains) == 0 {
		return true, ""
	}

	for _, domain := range domains {
		for _, blacklistedDomain := range *blacklist {
			if domain == blacklistedDomain {
				return false, domain
			} else {
				ok = true
			}
		}
	}

	return ok, ""
}
