package music

import (
	"errors"
	"fmt"
	"log"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/gavalink"
	"github.com/go-redis/redis/v8"
)

var ErrEmptyQueue = errors.New("empty queue")

var queue map[string][]*gavalink.TrackInfo

// stored in redis
var r *redis.Client

func getQueue(guildID string) ([]*gavalink.Track, error) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("lavalink:queue:%v", guildID)
	exists := r.Exists(r.Context(), key)
	existsResult, err := exists.Result()
	if err != nil {
		log.Println(err)
		existsResult = 0
	}

	if existsResult == 0 {
		return nil, nil
	}

	request := r.SMembers(r.Context(), key)
	result, err := request.Result()
	if err != nil {
		log.Println(err)
		return nil, err
	}

	if len(result) == 0 {
		return nil, ErrEmptyQueue
	}

	queue := make([]*gavalink.Track, 0)

	for _, track := range result {
		trackInfo, err := gavalink.DecodeString(track)
		if err != nil {
			log.Println(err)
			return nil, err
		}
		queue = append(queue, &gavalink.Track{
			Data: track,
			Info: *trackInfo,
		})
	}

	return queue, nil
}

func getNext(guildID string) (*gavalink.Track, error) {
	queue, err := getQueue(guildID)
	if err != nil || queue == nil {
		return nil, err
	}

	track := queue[0]

	removeQueue(guildID, track.Data)

	return track, nil
}

func addQueue(guildID, track string) (bool, error) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("lavalink:queue:%v", guildID)

	request := r.SAdd(r.Context(), key, track)
	result, err := request.Result()
	if err != nil {
		return false, err
	}

	if result == 0 {
		return false, nil
	}

	return true, nil

}

func removeQueue(guildID, track string) (bool, error) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("lavalink:queue:%v", guildID)

	request := r.SRem(r.Context(), key, track)
	result, err := request.Result()
	if err != nil {
		return false, err
	}

	if result == 0 {
		return false, nil
	}

	return true, nil

}

func removeQueueByIndex(guildID string, index int) (bool, error) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	tracks, err := getQueue(guildID)
	if err != nil || queue == nil {
		return false, err
	}

	for i, track := range tracks {
		if i == index {
			return removeQueue(guildID, track.Data)
		}
	}
	return false, errors.New("Not found")

}

func clearQueue(guildID string) (bool, error) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("lavalink:queue:%v", guildID)

	request := r.Del(r.Context(), key)
	result, err := request.Result()
	if err != nil {
		return false, err
	}

	if result == 0 {
		return false, nil
	}

	return true, nil
}
