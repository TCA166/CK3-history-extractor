# CK3-history-extractor

[![GitHub Pages Documentation](https://img.shields.io/badge/GitHub_Pages-Documentation-blue)](https://tca166.github.io/CK3-history-extractor/ck3_history_extractor)
[![Rust](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml/badge.svg)](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml)

A program designed for creating an encyclopedia of sorts containing your ck3 history  
It goes through the lineage of every player in the save file (meaning multiplayer save files also work) and extracts data about every character it encounters and their associates

## Usage

First you will need to decide what version of the program you want to use.
There are three potential versions of the program:

- **Release** - Every now and then I release a new GitHub release that has compiled binaries attached. You can simply download those binaries and use them.
- **Dev** - You can also just compile the program from source on your machine of choice. If you have no clue how the rust compilation system works just try running:

    ```sh
    make dependencies
    make cargo
    cp target/release/ck3_history_extractor ./ck3_history_extractor
    ```

- **Legacy** - a Python based script developed way back with no support, but you can access it [here](https://github.com/TCA166/CK3-history-extractor/releases/tag/v1)

After making your choice go find your save file in the format the program accepts.

### Obtaining the save file

In order to use any version of the program you are going to need a text based unzipped CK3 save file.
You can find your save files in your documents directory in ```%USERPROFILE%\Documents\Paradox Interactive\Crusader Kings III\save games\```.
If you are running the game with the default settings chances are the save file is compressed and if the save was for an ironman run then the save file is saved in their internal binary format.
You will need to decompress the save file and convert the save file to the text format then.

#### De-Ironmanning your save file

1. Backup your ironman savefile
2. Run the game in debug mode (add -debug_mode to launch options)
3. Load the Ironman savefile and let the game run for a month so that it saves the file in debug mode
4. Congrats! your save file in the savefile folder is now no longer encrypted

#### Decompressing your save file

The save file is compressed using zlib, meaning any ordinary decompression program is capable of decompressing the save file.
Just decompress it then as if it was an archive, then enter the new directory and copy the ```gamestate``` file inside.
That file is your actual file the program is expecting.

### Running the program

Depending on the version this might vary a bit, but the program expects to find the ```templates``` directory in your current working directory.
Make sure then you download it from this repository and place it so that the program will find it.
Then just run it and follow the prompts.
The program will create a new directory in your current working directory for each player within your save file.

## Output example

You can find output examples in the [examples](examples/) directory.
Just unzip the files contained within and witness the glory of my dynasty!

## Other similar tools

- [CK2-history-extractor](https://github.com/TCA166/CK2-history-extractor) is a tool like this for CK2

## Development status

Currently I'm rewriting the tool in Rust.
This means updated game support, faster speeds and no need for a Python installed.
Stay tuned for updates!

## License

This work is licensed under the MIT license.
The text of the license is fairly self explanatory, you can find it [here](./license.txt)
