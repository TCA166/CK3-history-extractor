# CK3-history-extractor

A program designed for creating an encyclopedia of sorts containing your ck3 history  
It goes through the lineage of every player in the savefile (meaning multiplayer savefiles also work) and extracts data about every character it encounters and their associates

## Usage

### How to Install

1. Ensure you have python 3 installed
2. Download the entire repository
3. Ensure your ck3 savefile is unencrypted (ironman save files are encrypted)
4. If your savegame is compressed (by default it is):
    1. Unzip your ck3 file as if it was an archive and get the gamestate file inside
    2. Add ".ck3" to your gamestate file - this is your actual savefile
5. Prepare your save file in the same directory as CK3_history_extractor.py
6. Make sure the template html files are in the same directory as CK3_history_extractor.py
7. Make sure the lookup file is in the same directory as CK3_history_extractor.py
8. Run CK3_history_extractor.py
9. Follow the prompts

### How to preview the end result

If you want to preview the end result download "TCA166 History.zip", unzip it and go to home.html

### Dependencies

- Python3
- Standard Python3 libraries
- A web Browser

### What to do if your character traits don't match up with the state in game

We need to regenerate trait_indexes.lookup file because it is no longer in sync with actual game trait ids (most likely due to an update)

1. Prepare 00_traits.txt in the same directory as CK3_lookup_gen.py (it can be found in steamapps\common\Crusader Kings III\game\common\traits)
2. Run CK3_lookup_gen.py

### How to decrypt ironman savefiles

1. Backup your ironman savefile
2. Run the game in debug mode (add -debug_mode to launch options)
3. Load the Ironman savefile and let the game run for a month so that it saves the file in debug mode
4. Congrats! your save file in the savefile folder is now no longer encrypted

## Optional Changes

For performance reasons and from my personal testing there is much detail in ck3 savefiles that is not needed.  
Hovewer more dedicated users may disagree with that assesment. As such here are instructions on how to use this program to it's fullest.  
**BE WARNED: performance will tank significantly**

### How to extract more distant characters

By default the program goes and searches characters one level deep. As in only characters directly associated.  
But you can change that easily since .py files are essentially text files.

1. Open CK3_history_extractor.py using any text editor (preferably an actual IDE or Np++)
2. Go to [line nr 665](./CK3_history_extractor.py#L665) (that's near the end) There you will find a line that says:

    ```Python
    lineage = gLineage(lineageData, data, Environment(loader=FileSystemLoader('')), limit=1)
    ```

3. Change the 1 to whatever number you want.

This will make the program export more general detail and characters

### How to extract more title data

By default the program searches only down when handling titles. This can be also changed to search without any constraints.

#### Enable detailed liege extraction

1. Open CK3_history_extractor.py using any text editor
2. Go to [line nr 74](./CK3_history_extractor.py#L74) There you will find a line that says:

    ```Python
    lookDownToggle = False
    ```

3. Change the False to True

#### Enable detailed vassal extraction

1. Open CK3_history_extractor.py using any text editor
2. Go to [line nr 104](./CK3_history_extractor.py#L104) There you will find a line that says:

    ```Python
    vassal = gTitle(vassal, allData, env, path, lookUp=False)
    ```

3. Change the False to True

## Other similar tools

- [CK2-history-extractor](https://github.com/TCA166/CK2-history-extractor) is a tool like this for CK2

## Development status

Currently there are still things to get done/things that could be improved. Most notably:

- Artifact extraction
- Rendering rework
- Code reformatting
- Graphical rework

Also rewriting the entire thing in C is a possibility now. Though not sure if that's a good idea. Performance would surely benefit, but by the nature of C the process would be a slog

## License

[![CCimg](https://i.creativecommons.org/l/by/4.0/88x31.png)](http://creativecommons.org/licenses/by/4.0/)  
This work (.py and .html files) is licensed under a [Creative Commons Attribution 4.0 International License](http://creativecommons.org/licenses/by/4.0/).  
