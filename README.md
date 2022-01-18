# WastedTime

This a bare-bones Proof of Concept to track time spent in different applications.

No cloud integration or any kind of network traffic is the main feature.

Windows only (but it builds on Linux for testing with bogus values).

## Building

```
cargo build --release
```

## Setup

Run the binary once to let it create the folder structure, then copy the config and create the DB.

  * `WastedTime.toml` goes into `%APPDATA%\\Roaming\art-core\\WastedTime\\config`
  * `WastedTime.sqlite` goes into `%APPDATA%\\Roaming\art-core\\WastedTime\\data`
  * load `resources/init.sql` into the db or create the `log` table manually

## Running

```
.\run-release.ps1
```

This will probably not work due to ps1 script execution being blocked, but whatever.

## Additional notes

If you don't build with `--release` it might be very spammy and the config and db paths are different, Linux example:

  * `$HOME/.config/wastedtime/WastedTime.dev.toml`
  * `$HOME/.local/share/wastedtime/WastedTime.dev.sqlite`

On the plus side, you're not polluting your "production" database.


## License

BSD-2-Clause