# Logging

Logging is a vital module that tells Black Mesa to log moderation-relevant events into a
designated channel.

## Configuration Example

```yaml
logging:
    channelID: 832311430019022851 # where to log
    includeActions: # included actions (mutually exclusive)
        - messageDelete
        - messageEdit
        - censor
    excludeActions: [] # excluded actions (mutually exclusive)
    timestamps: true # show timestamps
    timezone: UTC # the timezone for timestamps
    ignoredUsers: # ignored users
        - 206309860038410240
    ignoredChannels: # ignored channels
        - 832353063096287243
    newMemberThreshold: 600 # the threshold of which a member is considered new in seconds
```
