# CK3-history-extractor

[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/TCA166/CK3-history-extractor/total)](https://github.com/TCA166/CK3-history-extractor/releases/latest)
[![Example](https://img.shields.io/badge/GitHub_Pages-Output_Example-fuchsia)](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html)
[![Rust Documentation](https://img.shields.io/badge/GitHub_Pages-Documentation-blue)](https://tca166.github.io/CK3-history-extractor/ck3_history_extractor)
[![Tests](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml/badge.svg)](https://github.com/TCA166/CK3-history-extractor/actions/workflows/rust.yml)

A program for generating a wikipedia of your CK3 playthrough. It goes through
the lineage of every player in the save file and extracts data about every
character, title, religion, culture, etc. it encounters. It even renders cool
graphs that depict your history. You can preview how you can expect the result
to look like in your browser
[here](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html)

## Usage

You need to get the executable first. There are a few ways to do that.
The way I _recommend_, is to use the precompiled binaries that I release on
GitHub. This is the easiest way to get the program up and running.

You can also of course compile the program yourself. This is also openly
supported and I have a [makefile](./Makefile) that should help you with that. If
you are not familiar with compiling Rust programs, I recommend you check out the
[official Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html).
The program is written in Rust and should run on any platform that supports
Rust.

You can also use the legacy Python version of the program, but I do **not**
recommend it. You can find it
[here](https://github.com/TCA166/CK3-history-extractor/releases/tag/v1.0.0)

Having made your choice and downloaded the program, you then need to find
your save file of choice.

### Obtaining the save file

Your save files are most likely located in your documents directory, more
specifically in
`%USERPROFILE%\Documents\Paradox Interactive\Crusader Kings III\save games\`.
Having located the save file, ideally copy it over to the directory where you
placed the executable. This is not strictly necessary, but it will make your
life easier. Currently all save files are supported right out of the box.

### Running the program

You can run the program either from the command line or by double-clicking it.
If you run it without any arguments, the program will prompt you for the save
file path and the game path. Within the console interface, the program should
automatically pre-fill these paths based on your system. You can modify these
pre-filled paths if needed. Note that on some systems, this feature might not
work as expected.

**Important:** Ensure that the game path you provide points to the
`/game` subdirectory within your `Crusader Kings III` folder. Failing to
do so will result in errors when running the program.

The tool can also be used as a command line utility. Running the tool with the
`--help` argument will show you all the available options.

## Mod support

This program should work just fine on modded save files. Some specific aspects
may be represented inaccurately to the in game state, but that can be alleviated
with providing the tool with the mods you used to play the game. The tool should
automatically discover installed Steam mods and allow you to select them. If you want to use mods that are not detected through Now
naturally very invasive mods allowing for individual country de jure drift, or
using weird non standard title naming schemes might break the tool. If that
occurs please do let me know and I shall see what I can do.

### Load order

Internally all provided paths (those added via include and game-path) are
handled in the same way. Iteratively these paths are searched for localization
and a valid map definition, with the first path provided being searched almost
last. Once a valid map definition is found this definition is used and is
expected to be entirely valid. The game path is loaded last.

## Output example

You can find output examples in the [examples](examples/) directory. In order
to better showcase the tool, I have deployed the result, and you can view it
[here](https://tca166.github.io/CK3-history-extractor/TCA166's%20history/index.html).

## Other similar tools

- [CK2-history-extractor](https://github.com/TCA166/CK2-history-extractor) is a
  tool like this for CK2
- [pdx_unlimiter](https://github.com/crschnick/pdx_unlimiter) is a tool like
  this, but less focused on CK3 and less on the history aspect. It is more of a
  general purpose tool for extracting data from Paradox games.

## Problems

If you encounter any problems please do let me know. The best way to do that is
by creating a new issue on GitHub, this will let me track them better. This also
applies to feature suggestions and general feedback.

## Features

- Each in game entity will get it's own HTML page with data and links to other
  related entities. Characters will link to their Faith, Culture etc...
- Faith and Culture pages will display graphs showing the amount of deaths of
  their members through time, which allows for rough tracking of trends within
  your game
- Dynasty pages will display family trees
- Titles will display de facto maps of their extent if applicable and the game
  path is provided
- A timeline page with a graphic will display the lifespans of empires and
  notable events like conquests and falls of notable cities
- A timelapse gif will show the de jure growth of the importance of your line

## Development status

The tool has been rewritten using Rust. Currently I'm working on new features
and bugfixes. Be sure to star this repository or watch it to get regular updates
regarding the process. If you want to contribute feel free to do so! Plenty of
things to be done, just read [this](./CONTRIBUTING.md).

## License

This work is licensed under the MIT license. The text of the license is fairly
self explanatory, you can find it [here](./license.txt)
