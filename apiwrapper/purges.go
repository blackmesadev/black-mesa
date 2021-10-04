package apiwrapper

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/blackmesadev/black-mesa/structs"
)

func (c *APIClient) SendPurges(purges *structs.PurgeStruct) (*http.Response, error) {
	data, err := json.Marshal(purges)

	if err != nil {
		log.Println(err)
		return nil, err
	}

	reqPath := c.BaseURL + "/messages/purge/" + purges.UUID

	req, err := c.NewRequest("POST", reqPath, data)
	return c.Do(req)

}
