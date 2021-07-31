package util

import (
	"log"
	"net/http"
	"regexp"
	"strings"

	"github.com/PuerkitoBio/goquery"
	"github.com/blackmesadev/discordgo"
)

var domainsRegex = regexp.MustCompile(`(?:https?:\/\/)?(?:[^@\/\n]+@)?(?:www\.)?([^:\/\n]+)`)

func checkHeaders(url string, lookingFor string) bool {
	resp, err := http.Head(url)
	if err != nil {
		log.Println(err)
	} else {
		header := resp.Header.Get("Content-Type")
		if strings.HasPrefix(header, lookingFor) {
			return true
		}
	}

	return false
}

func CheckForImage(m *discordgo.Message) bool {

	if len(m.Embeds) > 0 {
		for _, embed := range m.Embeds {
			if embed.Image != nil || embed.Thumbnail != nil {
				return true
			}
		}
	}

	if len(m.Attachments) > 0 {
		for _, attachment := range m.Attachments {
			// We only need headers so dont waste our time downloaing whole file
			result := checkHeaders(attachment.URL, "image")
			if result {
				return true
			}
		}
	}

	// worst case scenario, download any web pages in the message:
	domains := domainsRegex.FindAllString(m.Content, -1)

	for _, domain := range domains {
		resp, err := http.Get(domain)
		if err != nil {
			log.Println(err)
			continue
		}

		doc, err := goquery.NewDocumentFromReader(resp.Body)
		if err != nil {
			log.Println(err)
			continue
		}

		foundInMeta := false
		doc.Find("meta").Each(func (i int, s *goquery.Selection) {
			tagName, exists := s.Attr("name")

			if exists {
				if tagName == "twitter:image" || tagName == "og:image" {
					content, exists := s.Attr("value")

					if !exists {
						content, exists = s.Attr("content")
						if !exists {
							log.Println("Meta tag without value?")
							return
						}
					}

					result := checkHeaders(content, "image")
					if result {
						foundInMeta = true
					}
				}
			}
		})

		if foundInMeta {
			return true
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
			result := checkHeaders(attachment.URL, "video")
			if result {
				return true
			}
		}
	}
	return false
}
