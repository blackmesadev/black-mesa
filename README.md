<p align="center" style="margin-bottom: 0px !important;"> 
    <img src="https://cdn.discordapp.com/attachments/387345753368166400/926217661245972500/blackmesa.png" width=200>
</p>
<h1 align="center" style="font-size:48px"> Black Mesa</h1>

<div align="center">

[![CodeFactor](https://www.codefactor.io/repository/github/blackmesadev/black-mesa/badge?s=14355c8033b3e76f4d0bf466d6726a52305a5d8b)](https://www.codefactor.io/repository/github/blackmesadev/black-mesa) ![Discord](https://img.shields.io/discord/832311430019022848) ![Lines of code](https://img.shields.io/tokei/lines/github.com/blackmesadev/black-mesa) ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/blackmesadev/black-mesa?sort=semver)

</div>


### Black Mesa is a Discord Moderation bot designed with **Performance**, **Reliability** and **Customization** in mind.
#

# Self Host Guide

## Docker

The prefered and supported method of running the bot is via [Docker](https://www.docker.com) with the provided `docker-compose.yml.example` file, you will be up and running in no time!

### Installation (Linux)
- You must first ensure you have up to date versions of [Docker](https://www.docker.com) and [Docker Compose](https://docs.docker.com/compose/install/)
- Download the source code for the project with `git clone https://github.com/blackmesadev/black-mesa.git`
- You will then need to `cp docker-compose.yml.example docker-compose.yml`
- Open the `docker-compose.yml` file with your preferred text editor and ensure you set the correct enviornment variables, they are listed accordingly. Do NOT forget to set a password for mongodb.
- Once your `docker-compose.yml` file is in a state that you are happy with it, simply run `docker-compose up --build -d` to build and bring up the container detached.

### Installation (Windows)
- why are you trying to run a bot on windows

# Usage

## MongoDB

Black Mesa uses [MongoDB](https://www.mongodb.com) as the database and will use it to read and write configuration data, actions and logs.

Black Mesa also uses [Redis](https://redis.io) for short term variable sharing such as keeping record of CPU/Memory Usage, Automod Data, etc.

Black Mesa configuration is in `docker-compse.yml`, this is where you will store your **Token**, **Mongo DB URI** and **Redis URI**

If you are using Docker you will need to `docker-compose build` again before the configuration updates.
