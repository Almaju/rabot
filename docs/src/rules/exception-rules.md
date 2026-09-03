# Exceptions

> The key: the exception is written somewhere. If it's only in your head,
> it's not an exception. It's just chaos with better intentions.

rabot's own mechanism has rules too. An allow comment must carry a reason and
name a rule that exists. And a file rabot cannot parse is reported rather
than skipped, so a syntax error never silently turns a check off.
