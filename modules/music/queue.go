package music

import (
	"context"

	gopherlink "github.com/damaredayo/gopherlink/proto"
)

func getQueue(guildID string) ([]*gopherlink.SongInfo, error) {
	queue, err := g.GetQueue(context.Background(), &gopherlink.QueueRequest{
		GuildId: guildID,
	})
	if err != nil {
		return nil, err
	}

	return queue.Songs, nil
}

func removeQueue(guildID, track string) error {
	// TODO: remove queue
	return nil
}

func removeQueueByIndex(guildID string, index int) error {
	// TODO: remove queue by index
	return nil
}

func clearQueue(guildID string) (bool, error) {
	// TODO: clear queue
	return false, nil
}
