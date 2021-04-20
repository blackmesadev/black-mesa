package discord

import (
	"fmt"
	"log"
	"regexp"
	"strings"

	"github.com/blackmesadev/black-mesa/automod"
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMessageCreate(s *discordgo.Session, mc *discordgo.MessageCreate) {

	var err error

	// Ignore all messages created by the Bot account itself
	if mc.Author.ID == s.State.User.ID {
		return
	}

	automod.Process(s, mc.Message)

	// Create Context struct that we can put various infos into
	ctx := &discordgo.Context{
		Content: strings.TrimSpace(mc.Content),
	}

	// Fetch the channel for this Message
	var c *discordgo.Channel
	c, err = s.State.Channel(mc.ChannelID)
	if err != nil {
		// Try fetchin  via REST API
		c, err = s.Channel(mc.ChannelID)
		if err != nil {
			log.Printf("unable to fetch Channel for Message, %s", err)
		} else {
			// Attempt to add this channel into our State
			err = s.State.ChannelAdd(c)
			if err != nil {
				log.Printf("error updating State with Channel, %s", err)
			}
		}
	}
	// Add Channel info into Context (if we successfully got the channel)
	if c != nil {
		if c.Type == discordgo.ChannelTypeDM {
			ctx.IsPrivate, ctx.IsDirected = true, true
		}
	}

	// Detect @name or @nick mentions
	if !ctx.IsDirected {

		// Detect if Bot was @mentioned
		for _, v := range mc.Mentions {

			if v.ID == s.State.User.ID {

				ctx.IsDirected, ctx.HasMention = true, true

				reg := regexp.MustCompile(fmt.Sprintf("<@!?(%s)>", s.State.User.ID))

				// Was the @mention the first part of the string?
				if reg.FindStringIndex(ctx.Content)[0] == 0 {
					ctx.HasMentionFirst = true
				}

				// strip bot mention tags from content string
				ctx.Content = reg.ReplaceAllString(ctx.Content, "")

				break
			}
		}
	}

	// Detect prefix mention
	prefix := config.GetPrefix(mc.GuildID)

	if !ctx.IsDirected && len(prefix) > 0 {

		// TODO : Must be changed to support a per-guild user defined prefix
		if strings.HasPrefix(ctx.Content, prefix) {
			ctx.IsDirected, ctx.HasPrefix, ctx.HasMentionFirst = true, true, true
			ctx.Content = strings.TrimPrefix(ctx.Content, prefix)
		}
	}

	// For now, if we're not specifically mentioned we do nothing.
	// later I might add an option for global non-mentioned command wors
	if !ctx.IsDirected {
		return
	}

	// Try to find the "best match" command out of the message.
	r, params, args := bot.Router.Match(ctx.Content)
	if r != nil {
		ctx.Fields = params
		r.Run(s, mc.Message, ctx, args)
		return
	}

	// If no command match was found, call the default.
	// Ignore if only @mentioned in the middle of a message
	if bot.Router.Default != nil && (ctx.HasMentionFirst) {
		// TODO: This could use a ratelimit
		// or should the ratelimit be inside the cmd handler?..
		// In the case of "talking" to another bot, this can create an endless
		// loop.  Probably most common in private messages.
		bot.Router.Default.Run(s, mc.Message, ctx, make([]string, 0))
	}
}
