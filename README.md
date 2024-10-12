# CK3-history-extractor

[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/TCA166/CK3-history-extractor/total)](https://github.com/TCA166/CK3-history-extractor/releases/latest)
[![Example](https://img.shields.io/badge/GitHub_Pages-Output_Example-fuchsia)](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html)
[![Rust Documentation](https://img.shields.io/badge/GitHub_Pages-Documentation-blue)](https://tca166.github.io/CK3-history-extractor/ck3_history_extractor)
[![Tests](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml/badge.svg)](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml)

A program for generating a wikipedia of your CK3 playthrough.
It goes through the lineage of every player in the save file and extracts data about every character, title, religion, culture, etc. it encounters.
It even renders cool graphs that depict your history.
You can preview how you can expect the result to look like in your browser [here](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html)

## Usage

First you will need to decide what version of the program you want to use.
There are three potential versions of the program:

- **Release** - Every now and then I release a new GitHub release that has compiled binaries attached.
    You can simply download those binaries and use them.
    For those unfamiliar with GitHub **[go here](https://github.com/TCA166/CK3-history-extractor/releases/latest)** and this is the version you should honestly use.
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
If you are running the game with the default settings chances are the save file is compressed and if the save was for an iron-man run then the save file is saved in their internal binary format.
You will need to convert the save file to the text format if that's the case.

#### De-iron-manning your save file

1. Backup your iron-man save file
2. Run the game in [debug mode](https://ck3.paradoxwikis.com/Console_commands#Enabling_debug_mode) (add -debug_mode to launch options)
3. Load the iron-man save file and let the game run for a month so that it saves the file in debug mode
4. Congrats! your save file in the save file folder is now no longer binary encoded

If you want to make sure that's actually the case, open the file in a text editor and see if you can see any gibberish or weird symbols.
If the save file has no gibberish(non ASCII characters) that means that the save file is ready to go.

#### Save file compression

The save file *may* be compressed.
This is no issue for the program however, it should automatically detect that and decompress the savefile.
A problem however, might arise if the compressed save file was also then iron-man encoded.
In such a case you will need to follow the steps laid out [in the previous section](#de-iron-manning-your-save-file).

### Running the program

You can run the program from the command line or just by double clicking it.
If you ran it with no arguments it will ask you for the path to the file and for the path to the game.
Within the console interface the program should automatically pre-enter the save file path and game path.
You can modify this pre-entered path of course and on some systems this feature might not function.
**It is very important that game path provided points to the ```/game``` subdirectory in your ```Crusader Kings III folder```**.
Otherwise you will receive errors when providing the path.
For users familiar with console environments here are details on that interface:

```sh
./ck3_history_extractor <save file path> <arg1> <arg2> ...
```

And here are the arguments that the utility accepts as of right now:

1. ```--internal``` forces the utility to use the internal templates.
2. ```--depth %d``` sets the maximum depth of the data to retrieve in the save file. The characters you played have depth=0, their relatives have depth=1 and so on.
3. ```--game-path %s``` shows the program where to find your ck3 localization data so that the pages can be completely accurate. Assuming you have the game installed via Steam you can do the following ```--game-path "*YOUR STEAM PATH*/steamapps/common/Crusader Kings III/game"```.
4. ```--no-vis``` disables all forms of visualization within the output
5. ```--language``` toggles which localization files shall be used if the game path is provided
6. ```--output %s``` changes where the output folder will be located
7. ```--include %s %s ...``` provides the program with a list of mod directories that the program should retrieve data from. These have higher priority than the game path
8. ```--dump``` makes the tool dump all the extracted data into a json file
9. ```--no-interaction``` disables automatic output opening and exit prompt

## Mod support

This program should work just fine on modded save files.
Some specific aspects may be represented inaccurately to the in game state, but that can be alleviated with providing the tool with the paths to the mods using the ```include``` argument.
Now naturally very invasive mods allowing for individual country de jure drift, or using weird non standard title naming schemes might break the tool.
If that occurs please do let me know and I shall see what I can do.

### Load order

Internally all provides paths (those added via include and game-path) are handled in the same way.
Iteratively these paths are searched for localization and a valid map definition, with the first path provided being searched almost last.
Once a valid map definition is found this definition is used and is expected to be entirely valid.
The game path is loaded last.

## Output example

You can find output examples in the [examples](examples/) directory.
Just unzip the files contained within and witness the glory of my dynasty!
Additionally I simply deployed the result to GitHub pages meaning you can preview the result [here](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html).

## Other similar tools

- [CK2-history-extractor](https://github.com/TCA166/CK2-history-extractor) is a tool like this for CK2

## Problems

If you encounter any problems please do let me know.
The best way to do that is by creating a new issue on GitHub, this will let me track them better.
This also applies to feature suggestions and general feedback.

## Features

- Each in game entity will get it's own HTML page with data and links to other related entities. Characters will link to their Faith, Culture etc...
- Faith and Culture pages will display graphs showing the amount of deaths of their members through time, which allows for rough tracking of trends within your game
- Dynasty pages will display family trees
- Titles will display de facto maps of their extent if applicable and the game path is provided
- A timeline page with a graphic will display the lifespans of empires and notable events like conquests and falls of notable cities
- A timelapse gif will show the de jure growth of the importance of your line

## Development status

The tool has been rewritten using Rust.
Currently I'm working on new features and bugfixes.
Be sure to star this repository or watch it to get regular updates regarding the process.
If you want to contribute feel free to do so!
Plenty of things to be done, just read [this](./CONTRIBUTING.md).

## License

This work is licensed under the MIT license.
The text of the license is fairly self explanatory, you can find it [here](./license.txt)
