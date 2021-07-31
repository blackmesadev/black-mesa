package util

import (
	"log"
	"net/http"
	"strings"

	"github.com/blackmesadev/discordgo"
)

func CheckForImage(m *discordgo.Message) bool {

	if len(m.Embeds) > 0 {
		for _, embed := range m.Embeds {
			if embed.Image != nil {
				return true
			}
		}
	}

	if len(m.Attachments) > 0 {
		for _, attachment := range m.Attachments {
			// We only need headers so dont waste our time downloaing whole file
			resp, err := http.Head(attachment.URL)
			if err != nil {
				log.Println(err)
			} else {
				header := resp.Header.Get("Content-Type")
				if strings.HasPrefix(header, "image") {
					return true
				}
			}
		}
	}
	return false
}

func CheckForVideo(m *discordgo.Message) bool {

	if len(m.Embeds) > 0 {
		for _, embed := range m.Embeds {
			if embed.Video != nil {
				return true
			}
		}
	}

	if len(m.Attachments) > 0 {
		for _, attachment := range m.Attachments {
			// We only need headers so dont waste our time downloaing whole file
			resp, err := http.Head(attachment.URL)
			if err != nil {
				log.Println(err)
			} else {
				header := resp.Header.Get("Content-Type")
				if strings.HasPrefix(header, "video") {
					return true
				}
			}
		}
	}
	return false
}
