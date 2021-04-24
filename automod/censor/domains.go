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

	for _, invite := range domains {
		for _, whitelistedDomain := range *whitelist {
			if invite == whitelistedDomain {
				ok = true
			} else {
				return false, invite
			}
		}
	}

	return ok, ""
}

func DomainsBlacklistCheck(m string, blacklist *[]string) (bool, string) {
	ok := true
	invites := domainsRegex.FindAllString(m, -1)

	if len(invites) == 0 {
		return true, ""
	}

	for _, invite := range invites {
		for _, blacklistedDomain := range *blacklist {
			if invite == blacklistedDomain {
				return false, invite
			} else {
				ok = true
			}
		}
	}

	return ok, ""
}
