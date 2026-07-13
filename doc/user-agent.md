# User Agent Requirements

It is important for us to know who is hitting our API so that we can reach out about any concerns or in extreme cases block the requests until issues can be resolved. Having a specific User-Agent may also allow us to exclude you from wider protection measures.

Your user-agent should contain the following info:

1. Your identity. Preferably a URL such as `example.com`. Allows us to get in touch.
1. A version number that increases. This allows us to block or specially handle old clients without blocking fixed clients. Ideally this number increases as you make releases, but it is also fine to just hardcode a number and increment it whenever you make important fixes.

For example: `example.com/1.23` is a great `User-Agent` if you own `example.com`.

**DO NOT** put end-user info into the User-Agent as it will be recorded for a short period of time for operational reasons.
