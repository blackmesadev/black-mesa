package spam

import "unicode"

func ProcessMaxUppercase(message string, percentageLimit float64, minimumLength int) bool {
	length := len(message)
	if length < minimumLength {
		return true
	}

	uppercase := 0

	for _, r := range message {
		if unicode.IsUpper(r) && unicode.IsLetter(r) {
			uppercase++
		}
	}

	percentage := (float64(uppercase) / float64(length)) * 100

	if percentage > percentageLimit {
		return false
	}

	return true
}