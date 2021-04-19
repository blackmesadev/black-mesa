package automod

import (
	"fmt"
	"regexp"
)

var domainsRegex = regexp.MustCompile(`^(?:https?:\/\/)?(?:[^@\/\n]+@)?(?:www\.)?([^:\/\n]+)`)

func DomainsWhitelistCheck(m string, whitelist *[]string) bool {
	ok := false

	domains := domainsRegex.FindAllString(m, -1)

	if len(domains) == 0 {
		return true
	}

	fmt.Println(domains)

	for _, invite := range domains {
		for _, whitelistedDomain := range *whitelist {
			if invite == whitelistedDomain {
				ok = true
			} else {
				return false
			}
		}
	}

	return ok
}

func DomainsBlacklistCheck(m string, blacklist *[]string) bool {
	ok := true
	invites := domainsRegex.FindAllString(m, -1)

	if len(invites) == 0 {
		return true
	}

	for _, invite := range invites {
		for _, blacklistedDomain := range *blacklist {
			if invite == blacklistedDomain {
				return false
			} else {
				ok = true
			}
		}
	}

	return ok
}
