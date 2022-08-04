# CK3-history-extractor
A program designed for creating an encyclopedia of sorts containing your ck3 history
<br>
How to Install:
<ol>
<li>Ensure you have python 3 installed</li>
<li>Download the entire repository</li>
<li>Prepare your unencrypted ck3 save file in the same directory as CK3_history_extractor.py (ironman save files are encrypted)</li>
<li>Make sure the template html files are in the same directory as CK3_history_extractor.py</li>
<li>Run CK3_history_extractor.py</li>
<li>Follow the prompts</li>
</ol>
If your character traits appear incorrect:
<ol>
<li>Prepare 00_traits.txt in the same directory as CK3_lookup_gen.py (it can be found in steamapps\common\Crusader Kings III\game\common\traits)</li>
<li>Run CK3_lookup_gen.py</li>
</ol>
If you want to preview the end result download "TCA166 History.zip", unzip it and go to home.html
<br>
<br>
How to decrypt ironman savefiles:
<ol>
<li>Backup your ironman savefile</li>
<li>Run the game in debug mode (add -debug_mode to launch options)</li>
<li>Load the Ironman savefile and let the game run for a month so that it saves the file in debug mode</li>
<li>Congrats! your save file in the savefile folder is now no longer encrypted</li>
</ol>
