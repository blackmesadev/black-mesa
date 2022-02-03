package logging

import (
	"strings"

	"github.com/blackmesadev/black-mesa/util"
)

func escapeBackticks(str string) string {
	return strings.ReplaceAll(str, "`", "`"+util.ZeroWidth)
}
