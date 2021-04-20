package moderation

import (
	"fmt"
	"regexp"
	"strconv"
	"time"
)

var snowflakeRegex = regexp.MustCompile(`([0-9]{17,18})`)

// seconds to regex for the string of it, makes iteration easier as you can use k as the multiplier for v
var timeRegex = map[int64]*regexp.Regexp{
	1:        regexp.MustCompile(`(\d+)s`),
	60:       regexp.MustCompile(`(\d+)m`),
	3600:     regexp.MustCompile(`(\d+)h`),
	86400:    regexp.MustCompile(`(\d+)d`),
	2628000:  regexp.MustCompile(`(\d+)mo`),
	31536000: regexp.MustCompile(`(\d+)y`),
}

func parseCommand(cmd string) ([]string, int64, string) {

	idList := snowflakeRegex.FindAllString(cmd, -1)
	duration, lastRegex := parseTime(cmd)

	reasonSearch := lastRegex.FindAllString(cmd, -1)

	reason := reasonSearch[len(reasonSearch)-1]

	fmt.Println(idList)
	fmt.Println(duration)
	fmt.Println(reasonSearch)
	fmt.Println(reason)

	return idList, duration, reason
}

// returns a int64 unix timestamp representative of when the punishment can be lifted
// also returns the last regex that works so that we can split the command with an already compiled
// regex which will give us the reason at the end! (if it doesnt its because the reason consisted of someone
// slamming their head against their keyboard for the reason and at that point i dont give a fuck)
func parseTime(strTime string) (int64, *regexp.Regexp) {
	var lastRegex *regexp.Regexp
	var unixTime int64

	unixTime = time.Now().Unix()

	for multiplier, regex := range timeRegex {
		timeValStrSlice := regex.FindAllString(strTime, -1)
		if timeValStrSlice != nil {
			timeVal, err := strconv.ParseInt(timeValStrSlice[0], 10, 32) // will be cast to uint32 so needs to be int32 at heart in an int64 body
			if err != nil {
				fmt.Println(err)
				unixTime = 4294967295
			}
			unixTime += timeVal * multiplier
			lastRegex = regex
		}
	}

	return unixTime, lastRegex
}
