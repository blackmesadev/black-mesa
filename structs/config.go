package structs

type FlatConfig struct {
	Token    string
	Mongo    MongoConfig
	Redis    RedisConfig
	Lavalink LavalinkConfig
	API      APIConfig
}

type MongoConfig struct {
	ConnectionString string
	Username         string
	Password         string
}

type RedisConfig struct {
	Host string
}

type LavalinkConfig struct {
	Host     string
	Password string
}

type APIConfig struct {
	Host  string
	Port  string
	Token string
}
