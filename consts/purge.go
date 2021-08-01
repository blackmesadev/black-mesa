package consts

type PurgeType string

const (
	PURGE_ALL          PurgeType = "all"
	PURGE_ATTACHEMENTS PurgeType = "attachments"
	PURGE_BOT          PurgeType = "bots"
	PURGE_IMAGE        PurgeType = "images"
	PURGE_STRING       PurgeType = "string"
	PURGE_USER         PurgeType = "users"
	PURGE_VIDEO        PurgeType = "videos"
)
