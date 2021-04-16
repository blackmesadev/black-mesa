# AutoMod

AutoMod is the primary module of Black Mesa. It is designed to take a load off of your human staff
and let Black Mesa make automated decisions regarding what users post based on configured filters.
Of course, it is not a substitute for staff members, but is meant to go along side them.

## Features

AutoMod contains the following features:

-   Censorship
    -   Zalgo filtering
    -   Invite filtering
        -   Invite and server ID whitelist and blacklists
    -   URL filtering
        -   Domain whitelist and blacklists
    -   String blocking
    -   Substring blocking
    -   Regex filters
-   Spam Filter
    -   Max messages in a duration
    -   Max mentions in a message
    -   Max links in a message
    -   Max attachments
    -   Max emojis
    -   Max newlines
    -   Max duplicate messages

## Configuration Example

### Basic Configuration

```yaml
automod:
    censorLevels:
        50: # the censorship configuration for level 50 users
            filterZalgo: true # filters out zalgo text
            filterInvites: true
            invitesWhitelist:
                - discord-developers # invite urls
                - 832311430019022848 # server ids
            domainBlacklist:
                - naughty-website.com
            blockedStrings: # block entire sentences...
                - this sentence is blocked!
            blockedSubstrings: # ... or single words.
                - naughty
            regex: # for more complicated filtering
                naughtyWord: 'n(au|ua)ghty!?'
        # etc. etc.
```

### Advanced Configuration

Let's say you wanted to use the same configuration for level 50 on multiple levels or channels. You
can do so with YAML anchors, designed to prevent constant copy/paste.

```yaml
censorConfig: &config
    filterZalgo: true # filters out zalgo text
    filterInvites: true
    invitesWhitelist:
        - discord-developers # invite urls
        - 832311430019022848 # server ids
    domainBlacklist:
        - naughty-website.com
    blockedStrings: # block entire sentences...
        - this sentence is blocked!
    blockedSubstrings: # ... or single words.
        - naughty
    regex: # for more complicated filtering
        naughtyWord: 'n(au|ua)ghty!?'

automod:
    censorLevels:
        50: *config
        70: *config
    censorChannels:
        832311430019022851: *config
    # etc. etc.
```
