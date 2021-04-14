package main

import (
	"encoding/json"
	"io/ioutil"
	"log"
	"os"
)

const VERSION string = ""

var bot *Bot

func main() {
	tokenBytes := GetToken()

	bot = &Bot{}
	err := json.Unmarshal(tokenBytes, bot)
	if err != nil {
		log.Fatalln(err)
	}
	startBot()

}

func GetToken() []byte {
	file, err := os.Open("token.json")
	if err != nil {
		log.Fatalln(err)
	}
	defer func() {
		if err = file.Close(); err != nil {
			log.Fatalln(err)
		}
	}()

	token, err := ioutil.ReadAll(file)
	if err != nil {
		log.Fatalln(err)
	}
	return token
}

func MessageSliceRemove(s []string, i int) []string {
	s[len(s)-1], s[i] = s[i], s[len(s)-1]
	return s[:len(s)-1]
}

func RemoveByString(s []string, r string) []string {
	for i, v := range s {
		if v == r {
			return append(s[:i], s[i+1:]...)
		}
	}
	return s
}
