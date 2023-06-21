# Sesser
Sesser is a small, light and quite fast database for storing session cookies and API keys at specified intervals.

## key features
- The database itself generates random strings of characters, which are then hashed using Sha256, so you don't have to generate them yourself
- Automatic database cleaning - the database cleans itself of "dead keys" whose time interval has already expired
- small binary file size - on Windows systems it is ~ 1MB, on Gnu/Linux ~ 6MB
