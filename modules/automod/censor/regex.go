package censor

import (
	"regexp"
	"strings"
	"time"
)

var regexCache = make(map[string]*regexp.Regexp)

// Return true if all is okay, return false if not.
func RegexCheck(m string, regexString string) (string, bool) {
	var err error

	regex, ok := regexCache[regexString]
	if !ok {
		regex, err = regexp.Compile(regexString)
		if err != nil {
			return "", true
		}
		regexCache[regexString] = regex
	}

	matchSlice := regex.FindAllString(m, -1)
	if len(matchSlice) > 0 {
		matches := strings.Join(matchSlice, ", ")
		return matches, false
	}

	return "", true
}

func StartFlushRegexCache() {
	// every 30 minutes, flush the cache
	ticker := time.NewTicker(time.Minute * 30)
	go func() {
		select {
		case <-ticker.C:
			regexCache = make(map[string]*regexp.Regexp)
		}
	}()
}
