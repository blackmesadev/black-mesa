# black-mesa

the main Discord bot service. listens to the Discord gateway for events, handles commands, automod, moderation actions, logging, and music playback via mesastream. built with a custom gateway implementation and uses the shared `bm-lib`.

## with Docker

```bash
docker build -t black-mesa .
```

start the container with:

```bash
docker run --rm \
	--name black-mesa \
	-e DISCORD_TOKEN="your_bot_token" \
	-e DATABASE_URL="postgres://user:pass@host.docker.internal:5432/blackmesa" \
	-e REDIS_URI="redis://host.docker.internal:6379" \
	-e MESASTREAM_BASE_URL="http://host.docker.internal:8070" \
	-e MESASTREAM_TOKEN="mesastream_bearer_token" \
	-e OTLP_ENDPOINT="http://host.docker.internal:4318/v1/traces" \
	black-mesa
```

## env vars

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `DISCORD_TOKEN` | Yes | N/A | Discord bot token. |
| `DATABASE_URL` | Yes | N/A | PostgreSQL connection string. |
| `REDIS_URI` | Yes | N/A | Redis connection string. |
| `MESASTREAM_BASE_URL` | Yes | N/A | Mesastream HTTP base URL. |
| `MESASTREAM_TOKEN` | Yes | N/A | Bearer token for mesastream API. |
| `OTLP_ENDPOINT` | Yes | N/A | OpenTelemetry OTLP endpoint. |
| `REDIS_PREFIX` | No | `bm` | Redis key prefix. |
| `OTLP_AUTH` | No | unset | Authorization header value for OTLP exporter. |
| `OTLP_ORGANIZATION` | No | unset | Optional org/tenant value for telemetry. |
| `SHARD_ID` | No | `0` | Shard ID for this instance. |
| `NUM_SHARDS` | No | `1` | Total number of shards. |

## sharding

for larger deployments, run multiple instances with different `SHARD_ID` values. each instance handles a subset of guilds determined by `(guild_id >> 22) % NUM_SHARDS == SHARD_ID`. all shards must share the same `DATABASE_URL` and `REDIS_URI`.
