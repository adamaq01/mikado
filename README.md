# Mikado

Little SDVX hook to submit your scores to a Tachi instance while you are playing.

## Installation

- Download the latest release from the [releases page](https://github.com/adamaq01/mikado/releases/latest)
- Extract the zip file into your SDVX installation folder
- When you start the game, inject the DLL into the process

## Tips

- The configuration file will be created in the same folder as the DLL at startup if it doesn't already exist
- You can configure some options (like the Tachi URL) by editing the `mikado.toml` file
- If you are using Spicetools, you can add the `-k mikado.dll` option or specify the DLL in the configuration tool to
  automatically inject it at startup

## License

MIT
