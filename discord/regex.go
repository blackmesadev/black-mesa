package discord

import (
	"fmt"
	"regexp"
)

type Regex struct {
	SelfMention *regexp.Regexp
}

func (b *Bot) InitRegex() {
	b.Regex.SelfMention = regexp.MustCompile(fmt.Sprintf("<@!?(%s)>", b.Session.State.User.ID))
}
