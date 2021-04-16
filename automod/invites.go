package automod

import (
	"regexp"
)

const inviteRegex = `discord(?:\.com|app\.com|\.gg)[\/invite\/]?(?:[a-zA-Z0-9\-]{2,32})`

func InvitesWhitelistCheck(m string, whitelist *[]string) bool {
	ok := false

	r := regexp.MustCompile(inviteRegex)

	invites := r.FindAllString(m, -1)

	if len(invites) == 0 {
		return true
	}

	for _, invite := range invites {
		for _, whitelistedInvite := range *whitelist {
			if invite == whitelistedInvite {
				ok = true
			} else {
				ok = false
			}
		}
	}

	return ok
}

func InvitesBlacklistCheck(m string, blacklist *[]string) bool {
	ok := true

	r := regexp.MustCompile(inviteRegex)

	invites := r.FindAllString(m, -1)

	if len(invites) == 0 {
		return true
	}

	for _, invite := range invites {
		for _, blacklistedInvite := range *blacklist {
			if invite == blacklistedInvite {
				ok = false
			} else {
				ok = true
			}
		}
	}

	return ok
}
