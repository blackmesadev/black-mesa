package moderation

import "regexp"

var snowflakeRegex = regexp.MustCompile(`([0-9]{17,18})`)
