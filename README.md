[![CodeFactor](https://www.codefactor.io/repository/github/blackmesadev/black-mesa/badge?s=14355c8033b3e76f4d0bf466d6726a52305a5d8b)](https://www.codefactor.io/repository/github/blackmesadev/black-mesa) ![Discord](https://img.shields.io/discord/832311430019022848) ![Lines of code](https://img.shields.io/tokei/lines/github.com/blackmesadev/black-mesa) ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/blackmesadev/black-mesa?sort=semver)

# Black Mesa

Black Mesa is a Discord Moderation bot designed with **Performance**, **Reliability** and **Customization** in mind.

This application is designed to be ran on Linux and has been tested to work on the following distros:
- Arch Linux (Version >2019.08.01)
- Manjaro (Version >18.1.0)
- Ubuntu (Version >16.04)
- Debian (Version >8)

# Installation

## Docker

The prefered and supported method of running the bot is via [Docker](https://www.docker.com) with the provided `docker-compose.yml` file, you will be up and running in no time!

## Build from Source

### Prerequisites
 - [Golang (>1.16)](https://golang.org)
 - [MongoDB](https://www.mongodb.com)
 - [Redis](https://redis.io)

### Installation
- First download the source code for the project with `git clone git@github.com:blackmesadev/black-mesa.git`
- Then build the code by first navigating to the newly made directory and building with the go compiler by running `go build .`
- Upon doing this a executable with the same name of the directory will be created, at which point the application can be ran with `./black-mesa` where `black-mesa` can be replaced with whatever the directory name is.

# Usage

## MongoDB

Black Mesa uses [MongoDB](https://www.mongodb.com) as the database and will use it to read and write configuration data, actions and logs.

Black Mesa also uses [Redis](https://redis.io) for short term variable sharing such as keeping record of CPU/Memory Usage, 

Black Mesa configuration is in `config.json`, this is where you will store your **Token**, **Mongo DB URI** and **Redis URI**

If you are using Docker you will need to `docker-compose build` again before the configuration updates.