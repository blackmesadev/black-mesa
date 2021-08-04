package music

import (
	"context"
	"fmt"
	"log"
	"net/url"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/lukasl-dev/waterlink"
)

var (
	conn waterlink.Connection
	req  waterlink.Requester
)

func LavalinkInit(config structs.LavalinkConfig) {
	var err error
	connOpts := waterlink.NewConnectOptions().WithUserID(config.Username).WithPassphrase(config.Password)
	reqOpts := waterlink.NewRequesterOptions().WithPassphrase(config.Password)

	httpHost, _ := url.Parse(fmt.Sprintf("http://%s", config.Host))
	wsHost, _ := url.Parse(fmt.Sprintf("ws://%s", config.Host))

	conn, err = waterlink.Connect(context.TODO(), *wsHost, connOpts)
	if err != nil {
		log.Fatalln(err)
	}

	req = waterlink.NewRequester(*httpHost, reqOpts)

	log.Println("Lavalink connected.")

}
