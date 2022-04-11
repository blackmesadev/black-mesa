package automod

import (
	"unicode"

	"golang.org/x/text/runes"
	"golang.org/x/text/transform"
	"golang.org/x/text/unicode/norm"
)

type RangeTable struct {
	Lo      uint16
	Hi      uint16
	Replace []uint16
}

var weirdCharactersRangeTable = []*RangeTable{
	{0x0110, 0x0111, []uint16{0x0044, 0x0064}},
	{0x0126, 0x0127, []uint16{0x0048, 0x0068}},
	{0x0131, 0x0133, []uint16{0x0049, 0x0049, 0x0049}}, // TODO: develop way to nicely return two runes for the case of 0x0132 and 0x0133 (Ĳ and ĳ)
	{0x0138, 0x0138, []uint16{0x004b}},
	{0x013f, 0x0142, []uint16{0x004c, 0x0049, 0x004c, 0x004e}},
	{0x0149, 0x014b, []uint16{0x004e, 0x004e, 0x004e}},
	{0x0166, 0x0167, []uint16{0x0054, 0x0054}},
}

func removeAccentsAndDiacritics(s string) string {
	t := transform.Chain(norm.NFD,
		runes.Remove(runes.In(unicode.Mn)),
		runes.Remove(runes.In(unicode.Diacritic)),
		norm.NFC)
	output, _, err := transform.String(t, s)
	if err != nil {
		return s
	}
	return output
}

func removeWeirdCharacters(s string) string {
	rs := []rune(s)
	for i, r := range rs {
		for _, v := range weirdCharactersRangeTable {
			var iteration int
			for z := v.Lo; z <= v.Hi; z++ {
				if rune(z) == r {
					rs[i] = rune(v.Replace[iteration])
				}
				iteration++
			}
		}
	}

	return string(rs)
}
