package censor

func ExtendedUnicodeCheck(s string) bool {
	for _, r := range []rune(s) {
		if r < 32 || r >= 127 {
			return false
		}
	}
	return true
}

func ObnoxiousUnicodeCheck(s string) bool {
	for _, r := range []rune(s) {
		if r >= 0x20dd && r <= 0x218f {
			return false
		}

		if r >= 0x2200 && r <= 0x22ff { // Mathematical Operators
			return false
		}

		if (r >= 0x2300 && r <= 0x2319) || (r > 0x231c && r < 0x23cd) || (r > 0x23d0 && r < 0x23e8) { // Miscellaneous Technical minus Emojis
			return false
		}

		if r >= 0x2400 && r <= 0x243f { // Control Pictures
			return false
		}

		if r >= 0x2460 && r <= 0x24ff && r != 0x24c2 { // Enclosed Alphanumerics minus Emoji
			return false
		}
		if r >= 0x1d400 && r <= 0x1d7ff { // Mathematical Alphanunmerics (a lot of annoying fonts)
			return false
		}
	}
	return true
}
