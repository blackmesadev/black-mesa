package apiwrapper

import (
	"bytes"
	"fmt"
	"net/http"

	"github.com/blackmesadev/black-mesa/structs"
)

type APIClient struct {
	BaseURL string `bson:"url" json:"url"`
	Token   string `bson:"token" json:"token"`

	httpClient *http.Client
}

var ApiInstance *APIClient

func InitAPI(apiConf structs.APIConfig) {
	ApiInstance = &APIClient{
		BaseURL:    fmt.Sprintf("http://%v:%v", apiConf.Host, apiConf.Port),
		Token:      apiConf.Token,
		httpClient: http.DefaultClient,
	}
}

// Initialize Client functions

func (c *APIClient) NewRequest(method string, path string, data []byte) (*http.Request, error) {
	var err error

	uri := c.BaseURL + path
	fmt.Println(uri)

	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest(method, uri, bytes.NewBuffer(data))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", c.Token)

	return req, nil
}

func (c *APIClient) Do(req *http.Request) (*http.Response, error) {
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	return resp, err
}
