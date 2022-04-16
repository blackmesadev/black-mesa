package untrustworthy

import (
	"regexp"
	"strings"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
)

// bool will be true if all is ok, false if not
func CheckUntrustworthy(msg string) (*structs.Untrustworthy, bool) {
	temp := &structs.Untrustworthy{
		FilterString: msg,
		Approved:     true,
	}

	u, err := db.GetSingleUntrustworthy(temp)

	if err != nil {
		return nil, true
	}

	// string filters

	if u.FilterString != "" {
		if strings.Contains(msg, u.FilterString) {
			return u, false
		}
	}

	// regex filters
	if u.FilterRegexString != "" {
		u.FilterRegexp, err = regexp.Compile(u.FilterRegexString)
		if err != nil {
			u.FilterRegexp = nil
		}
	}

	if u.FilterRegexp != nil {
		if u.FilterRegexp.MatchString(msg) {
			return u, false
		}
	}

	return nil, true
}
