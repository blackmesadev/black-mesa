bot := "black-mesa"
mongodb := "mongodb"
redis := "redis"

default:
    just --list

build:
    docker-compose build

start:
    docker-compose up -d

stop:
    docker-compose down

down: stop

run:
    docker-compose up

up: start

restart:
    docker-compose restart

logs service=bot:
    docker-compose logs -f {{service}}

clean:
    docker-compose down -v --remove-orphans
