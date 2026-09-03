# Errors

> Handle errors like any other value in your program. Every operation that
> can fail returns a discriminated union. No throws. No panics. Just honest
> return values that force you to confront reality.

Article: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

It is 3am and something between the request and the response is swallowing
an error. Six functions. Three nested catch blocks. Forty minutes of reading.
The problem started when somebody wrote a signature that promised a value and
could not keep the promise.

Three rules: panics in code that could have returned a `Result`, error types
that erase what went wrong, and the silent catch.
