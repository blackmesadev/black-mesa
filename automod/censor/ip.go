package censor

import (
	"net"
	"regexp"
	"strconv"
)

// Dumb regex - this is for cases like (2234.0.0.1) which would be errornously caught before.
// Rest of the checking is done in IPCheck(m string).
var ipRegex = regexp.MustCompile(`(\d+)\.(\d+)\.(\d+)\.(\d+)`)

// Return true if all is okay, return false if not.
func IPCheck(m string) bool {
	res := ipRegex.FindAllStringSubmatch(m, -1)

Outer:
	for i := range res {
		vals := make([]byte, 4)

		for j := range res[i] {
			val, err := strconv.ParseUint(res[i][j], 10, 64)

			if err != nil || val > 255 {
				// Fake IP
				continue Outer
			}

			vals[j] = byte(val)
		}

		ip := net.IPv4(vals[0], vals[1], vals[2], vals[3])

		if !ip.IsPrivate() && !ip.IsLoopback() && !ip.IsMulticast() && !ip.IsLinkLocalMulticast() && !ip.IsLinkLocalUnicast() {
			// This is a valid and public IP, so it should get filtered.
			return false
		}
	}

	return true
}
