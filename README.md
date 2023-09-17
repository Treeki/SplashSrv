# SplashSrv

_A proof-of-concept server for SEGA SPLASH! GOLF_

---

⚠️ This project is not affiliated with or endorsed by SEGA.

⛔️ This server is very incomplete and experimental. Don't expect to get a realistic gameplay experience out of it;
there are lots of missing features. The multiplayer code is not production-ready.

---

## Writeups

- **Part 1**: [They Made A Golf MMO With Sonic In it (Real!) (Not Clickbait!) (Only A Bit)](https://wuffs.org/blog/reviving-sega-splash-golf)
- **Part 2**: [Reviving Sega's forgotten golf MMO after 14 years](https://wuffs.org/blog/reviving-sega-splash-golf-part-2)
- **Part 3**: [Splash Golf Revival, Part 3: The Final Splash](https://wuffs.org/blog/reviving-sega-splash-golf-part-3)

---

## How To Run

I developed and tested this project with Rust 1.72 on an ARM MacBook, but newer versions should also work.

- Modify the `ip_address` in `src/login_server.rs` to point to the IP/hostname of the machine running the server
  - This is necessary to get past the Server Select screen
- Run with `cargo run`
- Create an account
  - Open the SQLite console with `sqlite3 splashsrv.db`
  - Enter: `INSERT INTO accounts (login_id, password) VALUES ("test", "asdf");`
- Run the game using [SplashHack](https://github.com/Treeki/SplashHack)
  - Log in using ID `test`, password `asdf` (or whatever else you put into the database!)
- Enjoy 2008's finest Pangya clone!

---

## Further Work

I don't expect to go further with this project as I'm about to start a new job, and I wanted to wrap up SplashSrv and
publish _something_ before then, even if it's very incomplete. With that in mind, here's a list of the improvements I
wanted to make.

- Better dev practices in the server code
  - Read configuration (e.g. the server hostname) from a file or environment variable
  - Hash passwords instead of storing them as plain-text in the database
  - Include some tests
  - Double-check packet definitions in `packets/mod.rs`
    - Some packets have incorrect bitfield definitions because I misunderstood how Deku handles bitfields, relative to
      Visual C++
  - Implement the various kinds of ping packets
- Complete user management and social features
  - Chat (`SEND_MESSAGE`)
  - Friends list and block list
  - Mail
  - Delivery
  - User titles
  - Colours (`ORD_COLOR_ELEMENT` and the like)
- Complete lobby/room management
  - Implement packet 24 (exiting a room)
  - Figure out and implement retiring from a game
  - Correctly remove players from a room when they disconnect
  - Allow room settings to be updated
  - Allow room leadership to be transferred
  - Invites
  - Figure out what happens when a room is empty (does it get deleted?)
- Improve the shop system
  - Figure out more realistic prices for items
  - Add all the equippable items to the shop
  - Remove invalid/incomplete items from the shop
  - Implement buying salon items
  - Implement buying new characters
  - Implement buying items with tickets
  - Implement renting caddies
  - Do something with the recycling shop
- Improve/complete gameplay
  - Use the room parameters when generating an `ORD_GAMESTART` packet
    - Allow different courses to be played
    - Pick random holes, weather, etc.
  - Implement item usage
  - Implement caddy usage
  - Implement 'growth' (gaining experience and levels)
  - Implement quick matching (ranked 1v1 battles)
  - Store and track the different kinds of records
  - Test and fix Competition Mode
