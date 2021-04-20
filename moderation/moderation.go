package moderation

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"time"
)

var snowflakeRegex = regexp.MustCompile(`([0-9]{17,18})`)

var numberRegex = regexp.MustCompile(`[0-9]*[.]?[0-9]+`)

// seconds to regex for the string of it, makes iteration easier as you can use k as the multiplier for v
var timeRegex = map[int64]*regexp.Regexp{
	1:        regexp.MustCompile(`(\d+)s`),
	60:       regexp.MustCompile(`(\d+)m`),
	3600:     regexp.MustCompile(`(\d+)h`),
	86400:    regexp.MustCompile(`(\d+)d`),
	604800:   regexp.MustCompile(`(\d+)w`),
	2628000:  regexp.MustCompile(`(\d+)mo`),
	31536000: regexp.MustCompile(`(\d+)y`),
}

func parseCommand(cmd string) ([]string, int64, string) {
	var reason string

	idList := snowflakeRegex.FindAllString(cmd, -1)

	params := snowflakeRegex.Split(cmd, -1)

	if params[len(params)-1][:1] == ">" {
		reason = params[len(params)-1][1:]
	} else {
		reason = params[len(params)-1]
	}

	durationStr := strings.Fields(reason)[0]
	duration := parseTime(durationStr)

	reason = strings.ReplaceAll(reason, durationStr, "")

	reason = strings.TrimSpace(reason)

	fmt.Println(idList)
	fmt.Println(duration)
	fmt.Println(params)
	fmt.Println(reason)

	return idList, duration, reason
}

// returns a int64 unix timestamp representative of when the punishment can be lifted
// also returns the last regex that works so that we can split the command with an already compiled
// regex which will give us the reason at the end! (if it doesnt its because the reason consisted of someone
// slamming their head against their keyboard for the reason and at that point i dont give a fuck)
func parseTime(strTime string) int64 {
	var unixTime int64

	unixTime = time.Now().Unix()

	for multiplier, regex := range timeRegex {
		timeValStrSlice := regex.FindAllString(strTime, -1)
		if timeValStrSlice != nil {
			timeVal, err := strconv.ParseInt(numberRegex.FindAllString(timeValStrSlice[0], 1)[0], 10, 32) // will be cast to uint32 so needs to be int32 at heart in an int64 body
			if err != nil {
				if strings.Contains(err.Error(), "strconv.ParseInt: parsing") {
					return 0
				}
			}
			unixTime += timeVal * multiplier
		}
	}

	return unixTime
}
