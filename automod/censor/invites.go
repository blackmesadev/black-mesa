package censor

import (
	"regexp"
)

var ggInviteRegex = regexp.MustCompile(`discord\.gg[\/invite\/]?(?:[a-zA-Z0-9\-]{2,32})`)
var stdInviteRegex = regexp.MustCompile(`discord(?:\.com|app\.com)\/invite\/?(?:[a-zA-Z0-9\-]{2,32})`)
var cdnRegex = regexp.MustCompile(`(cdn\.discord(?:\.com|app\.com))`)

func InvitesWhitelistCheck(m string, whitelist []string) (bool, string) {
	ok := false

	invites := stdInviteRegex.FindAllString(m, -1)
	ggInvites := ggInviteRegex.FindAllString(m, -1)
	invites = append(invites, ggInvites...)

	cdnCheck := cdnRegex.FindAllString(m, -1)
	if len(cdnCheck) >= 1 {
		return true, ""
	}

	if len(invites) == 0 {
		return true, ""
	}

	for _, invite := range invites {
		for _, whitelistedInvite := range whitelist {
			if invite == whitelistedInvite {
				ok = true
			} else {
				return false, invite
			}
		}
	}

	return ok, ""
}

func InvitesBlacklistCheck(m string, blacklist []string) (bool, string) {
	ok := true

	invites := stdInviteRegex.FindAllString(m, -1)
	ggInvites := ggInviteRegex.FindAllString(m, -1)
	invites = append(invites, ggInvites...)

	cdnCheck := cdnRegex.FindAllString(m, -1)
	if len(cdnCheck) >= 1 {
		return true, ""
	}

	if len(invites) == 0 {
		return true, ""
	}

	for _, invite := range invites {
		for _, blacklistedInvite := range blacklist {
			if invite == blacklistedInvite {
				return false, invite
			} else {
				ok = true
			}
		}
	}

	return ok, ""
}
