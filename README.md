# Mikado

Little SDVX hook to submit your scores to a Tachi instance while you are playing.

## Features

- Submit scores to a Tachi instance after each song
- Submit courses results to a Tachi instance
- Display your Tachi PBs scores in game as cloudlink (konaste) scores

## Installation

- Download the latest release from the [releases page](https://github.com/adamaq01/mikado/releases/latest)
- Put it in your game installation root directory (optional: create and edit the config file to set your API key)
- When you start the game, inject the DLL into the process

## Tips

- The configuration file will be created in the same folder as the DLL at startup if it doesn't already exist
- You can configure some options (like the Tachi URL) by editing the `mikado.toml` file
- If you are using Spicetools, you can add the `-k mikado.dll` option or specify the DLL in the configuration tool to
  automatically inject it at startup

## License

MIT
