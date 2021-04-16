# Bulletin

Bulletin is a module designed to make your moderation system transparent. Every set day, a message
will be posted containing moderation statistics from the last 7 days, including kicks, bans
(with a separate statistic for AutoMod bans) and strikes. As no sensitive information is stated in
the bulletin, you can make it public with no consequence.

## Enable

To enable Bulletin, run `!bulletin <channel> <day>` (e.g. `!bulletin #mod-bulletin monday`). This
will tell Black Mesa to post its moderation bulletin in a certain channel on a certain day.

!!! tip
Black Mesa will post the bulletin based on the UTC timezone. As such it may be offset for you
or your staff members.
