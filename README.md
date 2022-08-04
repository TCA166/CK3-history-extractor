# CK3-history-extractor
<h3>A program designed for creating an encyclopedia of sorts containing your ck3 history</h3>
<br>
<h4>How to Install:</h4>
<ol>
<li>Ensure you have python 3 installed</li>
<li>Download the entire repository</li>
<li>Prepare your unencrypted ck3 save file in the same directory as CK3_history_extractor.py (ironman save files are encrypted)</li>
<li>Make sure the template html files are in the same directory as CK3_history_extractor.py</li>
<li>Make sure the lookup file is in the same directory as CK3_history_extractor.py</li>
<li>Run CK3_history_extractor.py</li>
<li>Follow the prompts</li>
</ol>
<h4>If your character traits appear incorrect:</h4>
We need to regenerate trait_indexes.lookup file because it is no longer in sync with actual game trait ids (most likely due to an update)
<ol>
<li>Prepare 00_traits.txt in the same directory as CK3_lookup_gen.py (it can be found in steamapps\common\Crusader Kings III\game\common\traits)</li>
<li>Run CK3_lookup_gen.py</li>
</ol>
<h4>How to decrypt ironman savefiles:</h4>
<ol>
<li>Backup your ironman savefile</li>
<li>Run the game in debug mode (add -debug_mode to launch options)</li>
<li>Load the Ironman savefile and let the game run for a month so that it saves the file in debug mode</li>
<li>Congrats! your save file in the savefile folder is now no longer encrypted</li>
</ol>
<h4>How to make the program go to as much detail as you want</h4>
By default the program goes and searches characters one level deep. As in only characters directly associated.
But you can change that easily since .py files are essentially text files.
<ol>
<li>Open CK3_history_extractor.py using any text editor (preferably an actual IDE or Np++)</li>
<li>Go to line nr 582 (that's near the end)</li>
There you will find a line that says <code>lineage = gLineage(lineageData, allChars, data, Environment(loader=FileSystemLoader('')), 1)</code>
<li>Change the 1 to whatever number you want. BE WARNED: this will double parsing time(if you change it to 2)</li>
This will make the program export more detail and characters
</ol>
<h5>If you want to preview the end result download "TCA166 History.zip", unzip it and go to home.html</h5>
<br>
<a rel="license" href="http://creativecommons.org/licenses/by/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by/4.0/88x31.png" /></a><br />This work is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by/4.0/">Creative Commons Attribution 4.0 International License</a>.
