package misc

import "regexp"

var UserIdRegex = regexp.MustCompile(`^(?:<@!?)?(\d+)>?$`)

var RoleIdRegex = regexp.MustCompile(`^(?:<@&!?)?(\d+)>?$`)

var SnowflakeRegex = regexp.MustCompile(`([0-9]{17,18})`)

var NumberRegex = regexp.MustCompile(`[0-9]*[.]?[0-9]+`)

var UuidRegex = regexp.MustCompile(`\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b`)

// seconds to regex for the string of it, makes iteration easier as you can use k as the multiplier for v
var TimeRegex = map[int64]*regexp.Regexp{
	1:        regexp.MustCompile(`(\d+)s`),
	60:       regexp.MustCompile(`(\d+)m`),
	3600:     regexp.MustCompile(`(\d+)h`),
	86400:    regexp.MustCompile(`(\d+)d`),
	604800:   regexp.MustCompile(`(\d+)w`),
	2628000:  regexp.MustCompile(`(\d+)mo`),
	31536000: regexp.MustCompile(`(\d+)y`),
}
