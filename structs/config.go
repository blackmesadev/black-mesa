package structs

type FlatConfig struct {
	Token string      `json:"token"`
	Mongo MongoConfig `json:"mongo"`
	Redis RedisConfig `json:"redis"`
}

type MongoConfig struct {
	ConnectionString string `json:"connectionString"`
	Username         string `json:"username,omitempty"`
	Password         string `json:"password,omitempty"`
}

type RedisConfig struct {
	Host string `json:"host"`
}