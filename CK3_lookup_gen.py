import re
from datetime import date

with open ('00_traits.txt', "r", encoding='UTF-8') as traitsFile:
    data=traitsFile.read()
    traits = re.findall(r'\n(?!\t)(.*?) = {', data, re.ASCII)
    #print(traits)
    with open('trait_indexes.lookup', 'w') as lookupFile:
        for line in traits:
            lookupFile.write('%s\n' % line)
        lookupFile.write('This file was generated to emulate og ck3 trait id lookup table on ' + str(date.today()) + ' . This is important because it might be outdated')
print('Done')

