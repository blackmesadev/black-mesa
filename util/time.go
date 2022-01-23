package util

import (
	"fmt"
	"strconv"
	"strings"
	"time"
)

// returns a int64 unix timestamp representative of a time string
func ParseTime(strTime string) int64 {
	var unixTime int64

	unixTime = time.Now().Unix()

	for multiplier, regex := range TimeRegex {
		timeValStrSlice := regex.FindAllString(strTime, -1)
		if timeValStrSlice != nil {
			timeVal, err := strconv.ParseInt(NumberRegex.FindAllString(timeValStrSlice[0], 1)[0], 10, 32) // will be cast to uint32 so needs to be int32 at heart in an int64 body
			if err != nil {
				fmt.Println(err)
				if strings.Contains(err.Error(), "strconv.ParseInt: parsing") {
					return 0
				}
			}
			unixTime += timeVal * multiplier
		}
	}

	// fallback
	if unixTime == time.Now().Unix() {
		return 0
	}

	return unixTime
}
