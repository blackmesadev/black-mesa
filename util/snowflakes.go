package util

import (
	"strconv"
	"time"
)

func SnowflakeToTimestamp(userId string) time.Time {
	var createdUnix int
	snowflakeInt, err := strconv.Atoi(userId)
	if err != nil {
		createdUnix = 0
	}

	createdUnix = snowflakeInt>>22 + 1420070400000 // bitsift and add discord epoch for unix timestamp

	return time.UnixMilli(int64(createdUnix))
}
