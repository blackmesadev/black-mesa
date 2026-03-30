default:
    @just --list

services:
    @docker compose config --services

dev:
    @echo "Dev environment ready"

run *services: dev
    #!/usr/bin/env bash
    if [ -z "{{services}}" ]; then
        docker compose up --build -d
        docker compose logs -f black-mesa mesastream
    else
        docker compose up --build -d {{services}}
        docker compose logs -f {{services}}
    fi

start *services: dev
    #!/usr/bin/env bash
    if [ -z "{{services}}" ]; then
        docker compose up --build -d
    else
        docker compose up --build -d {{services}}
    fi

stop:
    docker compose down

clean:
    docker compose down -v
    @echo "Cleaned up containers, networks, and volumes"

logs *service:
    #!/usr/bin/env bash
    if [ -z "{{service}}" ]; then
        docker compose logs -f black-mesa mesastream mesa-api
    else
        docker compose logs -f {{service}}
    fi

build *services:
    #!/usr/bin/env bash
    if [ -z "{{services}}" ]; then
        docker compose build
    else
        docker compose build {{services}}
    fi

restart *service:
    #!/usr/bin/env bash
    if [ -z "{{service}}" ]; then
        docker compose restart
    else
        docker compose restart {{service}}
    fi

ps:
    docker compose ps

exec service *command:
    docker compose exec {{service}} {{command}}

shell service:
    docker compose exec {{service}} /bin/sh

pull:
    docker compose pull

stats:
    docker stats $(docker compose ps -q)

reload *services:
    #!/usr/bin/env bash
    if [ -z "{{services}}" ]; then
        docker compose up --build -d
        docker compose logs -f
    else
        docker compose up --build -d {{services}}
        docker compose logs -f {{services}}
    fi

fmt *services:
    #!/usr/bin/env bash
    for service in {{services}}; do
        case $service in
            black-mesa)
                pushd black-mesa > /dev/null
                cargo fmt
                popd > /dev/null
                ;;
            api)
                pushd api > /dev/null
                cargo fmt
                popd > /dev/null
                ;;
            mesastream)
                pushd mesastream > /dev/null
                cargo fmt
                popd > /dev/null
                ;;
            *)
            ;;
        esac
    done
