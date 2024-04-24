import re
from jinja2 import Environment, FileSystemLoader
import os
import time
import linecache
import sys
import traceback

#I outsourced all save file interpretation to the classes to make this .py file usable as a library

knownChars = {}
knownDyns = {}
knownCuls = {}
knownFaiths = {}
knownTitles = {}
knownHouses = {}

#(.*?) - narrow search
#.+? - wide search

TEMPLATE_PATH = 'src/templates'

class gMem:
    def __init__(self, memid:str, allData:str) -> None:
        self.memId = memid
        data = findMemData(memid, allData)
        findType = re.findall(r'type="(.*?)"', data, re.S)[0]
        self.type = gameStringToRead(findType)
        try:
            findDate = re.findall(r'creation_date=(.*?)\n', data, re.S)
            self.date = findDate[0]
        except IndexError:
            self.date = ''
        if 'participants' in data:
            self.participants = {}
            findParticipants = re.findall(r'participants={(.*?)\n\t\t\t}', data, re.S)[0].replace('\t','')
            lines = findParticipants.split('\n')[1:]
            for line in lines:
                fields = line.split('=')
                if fields[1] in knownChars.keys():
                    self.participants[gameStringToRead(fields[0])] = knownChars[fields[1]]
                else:
                    charData = findCharData(fields[1], allData)
                    findName = re.findall(r'first_name="(.*?)"', charData, re.S)
                    self.participants[gameStringToRead(fields[0])] = gameStringToRead(findName[0].replace('_', '')) 

class gTitle:
    def __init__(self, titleid:str, allData:str, env:Environment, path:str, lookUp:bool = True, lookDown:int = True) -> None:
        global knownTitles
        self.titleid = titleid
        rawData = findTitleData(titleid, allData)
        findKey = re.findall(r'key="(.*?)"', rawData, re.S)
        self.key = findKey[0]
        self.name = getTitleName(titleid, allData)
        knownTitles[titleid] = self
        if 'b_' not in self.key:
            if 'history=' in rawData:
                findHistory = re.findall(r'history={ (.*?) }', rawData, re.S)[0]
                historyData = findHistory.split(' ')
                self.history = {}
                for element in historyData:
                    fields = element.split('=', 1)
                    #here we dont actually create new char objects for characters we haven't yet found. This is a optimisation feature
                    #If we did go and extract data on each holder we would have a lot of unnecessary data due to how the gChar class works
                    if '{' not in fields[1]:
                        fields[1] = getChrNameOrObj(fields[1], allData)
                        self.history[fields[0]] = [fields[1], 'gained']
                    else:
                        type = re.findall(r'type=(.*?)\n', fields[1], re.S)[0]
                        if type == 'destroyed':
                            holder = ''
                        else:
                            holder = re.findall(r'holder=(.*?)\n', fields[1], re.S)[0]
                            holder = getChrNameOrObj(holder, allData)
                        self.history[fields[0]] = [holder, gameStringToRead(type)]
            lookDownToggle = False
            if 'de_jure_liege' in rawData:
                findJure = re.findall(r'de_jure_liege=(.*?)\n', rawData, re.S)[0]
                if findJure in knownTitles.keys():
                    findJure = knownTitles[findJure]
                else:
                    if lookUp:
                        findJure = gTitle(findJure, allData, env, path, lookUp=True, lookDown=lookDownToggle)
                    else:
                        findJure = getTitleName(findJure, allData)
                self.de_jure = findJure
            if 'de_facto_liege' in rawData:
                findFacto = re.findall(r'de_facto_liege=(.*?)\n', rawData, re.S)[0]
                if findFacto in knownTitles.keys():
                    findFacto = knownTitles[findFacto]
                else:
                    if lookUp:
                        findFacto = gTitle(findFacto, allData, env, path, lookUp=True, lookDown=lookDownToggle)
                    else:
                        findFacto = getTitleName(findFacto, allData)
                self.de_facto = findFacto
            if 'de_jure_vassals=' in rawData:
                findVassals = re.findall(r'de_jure_vassals={ (.*?) }', rawData, re.S)[0]
                vassals = findVassals.split(' ')
                self.vassals = []
                for vassal in vassals:
                    if vassal in knownTitles.keys():
                        vassal = knownTitles[vassal]
                    else:
                        if lookDown:
                            vassal = gTitle(vassal, allData, env, path, lookUp=False)
                        else:
                            vassal = getTitleName(vassal, allData)
                    self.vassals.append(vassal)
        knownTitles[titleid] = self
        if env != False and 'b_' not in self.key:
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'titleTemplate.html'))
            output = template.render(title = self)
            f = open(os.path.join(path, 'titles', titleid + '.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()

class gFaith:
    def __init__(self, faid:str, allData:str, env:Environment, path:str) -> None:
        global knownTitles
        self.faid = faid
        faiData = re.findall(r'\n\tfaiths={(.*?)\n}', allData, re.S)[0]
        rawData = re.findall(r'\n\t\t%s={(.*?)\n\t\t}' % faid, faiData, re.S)[0]
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        if len(findName) > 0:
            self.name = findName[0].capitalize()
        else:
            findTag = re.findall(r'tag="(.*?)"', rawData, re.S)
            self.name = gameStringToRead(findTag[0]).capitalize()
        findFervor = re.findall(r'fervor=(.*?)\n', rawData, re.S)
        self.fervor = findFervor[0]
        findTenets = re.findall(r'doctrine="tenet_(.*?)"', rawData, re.S)
        self.tenets = []
        for tenet in findTenets:
            self.tenets.append(gameStringToRead(tenet))
        findDoctrines = re.findall(r'doctrine="(.*?)"', rawData, re.S)
        basic = findDoctrines[3:21]
        others = findDoctrines[21:len(findDoctrines)]
        doctrines = []
        for doctrine in basic + others:
            doctrines.append(gameStringToRead(doctrine))
        self.doctrines = doctrines
        findReligion = re.findall(r'religion=(.*?)\n', rawData, re.S)
        self.religion = findReligion[0]
        findHead =  re.findall(r'religious_head=(.*?)\n', rawData, re.S)
        if len(findHead) > 0:
            try:
                if findHead[0] in knownTitles.keys():
                    self.head = knownTitles[findHead[0]]
                else:
                    self.head = gTitle(findHead[0], allData, env, path)
            except IndexError:
                pass
        knownFaiths[faid] = self
        if env != False:
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'faithTemplate.html'))
            output = template.render(faith = self)
            f = open(os.path.join(path, 'faiths', faid + '.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()

class gCulture:
    def __init__(self, culid:str, allData:str, env:Environment, path:str) -> None:
        global knownCuls
        self.culid = culid
        culData = re.findall(r'\n\tcultures={(.*?)\n}', allData, re.S)[0]
        rawData = re.findall(r'\n\t\t%s={(.*?)\n\t\t}' % culid, culData, re.S)[0]
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        self.name = findName[0].replace('_','').capitalize()
        findDate = re.findall(r'created=(.*?)\n', rawData, re.S)
        if len(findDate) > 0:
            self.date = findDate[0]
        findEthos = re.findall(r'ethos="(.*?)"', rawData, re.S)
        self.ethos = gameStringToRead(findEthos[0]).capitalize()
        findHeritage = re.findall(r'heritage="(.*?)"', rawData, re.S)
        self.heritage = gameStringToRead(findHeritage[0]).capitalize()
        findLanguage = re.findall(r'language="(.*?)"', rawData, re.S)
        self.language = gameStringToRead(findLanguage[0]).capitalize()
        findParents = re.findall(r'parents={(.*?)}', rawData, re.S)
        self.parents = []
        if len(findParents) > 0:
            parents = findParents[0].split(' ')[1:-1]
            for parent in parents:
                if parent in knownCuls.keys():
                    self.parents.append(knownCuls[parent])
                else:
                    self.parents.append(gCulture(parent, allData, env, path))
        findTraditions = re.findall(r'traditions={(.*?)}', rawData, re.S)
        traditions = findTraditions[0].split(' ')[1:-1]
        self.traditions = []
        for tradition in traditions:
            self.traditions.append(gameStringToRead(tradition))
        findMartial = re.findall(r'martial_custom="(.*?)"', rawData, re.S)
        self.martial = gameStringToRead(findMartial[0])
        knownCuls[culid] = self
        if env != False:
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'cultureTemplate.html'))
            output = template.render(culture = self)
            f = open(os.path.join(path, 'cultures', culid + '.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()



class gDynn:
    def __init__(self, dynid:str, allData:str, env:Environment, path:str, house:bool = True) -> None:
        if house:
            rawData = findHouseData(dynid, allData)
            knownDyns[dynid] = self
        else:
            rawData = findDynastyData(dynid, allData)
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        try:
            self.name = gameStringToRead(findName[0])
        except:
            findName = re.findall(r'key="(.*?)"', rawData, re.S)
            self.name = gameStringToRead(findName[0].replace('dynn_','').replace('_',''))
        self.dynid = dynid
        try:
            findDate = re.findall(r'found_date=(.*?)\n', rawData, re.S)
            self.date = findDate[0]
        except IndexError:
            self.date = 'Time immemorial...'
        if house:
            findParent = re.findall(r'\tdynasty=(.*?)\n', rawData, re.S)
            parentId = findParent[0]
            self.parent = gDynn(parentId, allData, env, path, False)
            self.members = allData.count('dynasty_house=%s' % dynid)
        else:
            self.houses = allData.count('dynasty=%s' % dynid)
            findprestige_tot = re.findall(r'accumulated=(.*?)\n', rawData, re.S)[0]
            self.prestige_tot = findprestige_tot
            findPrestige = re.findall(r'currency=(.*?)\n', rawData, re.S)[0]
            self.prestige = findPrestige
            try:
                findPerks = re.findall(r'perk={ (.*?) }', rawData, re.S)[0]
                perks = findPerks.split(' ')
                perkDict = {}
                for perk in perks:
                    perk = perk.replace('"', '')
                    key = gameStringToRead(perk[:-2])
                    val = int(perk[len(perk) - 1:])
                    if key in perkDict.keys():
                        if val > perkDict[key]:
                            perkDict[key] = val
                    else:
                        perkDict[key] = val
                self.perks = perkDict
            except IndexError:
                self.perks = {}
        try:
            findHistorical = re.findall(r'historical={(.*?)}', rawData, re.S)
            historicalLeaders = findHistorical[0].split(' ')[1:-1]
        except IndexError:
            historicalLeaders = []
        leaders = []
        for leader in historicalLeaders:
            leaders.append(getChrNameOrObj(leader, allData))
        self.leaders = leaders
        if house:
            knownDyns[dynid] = self
        if env != False:
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'dynastyTemplate.html'))
            absFilePath = os.path.realpath(__file__) 
            locPath = os.path.dirname(absFilePath) + '\\' + path 
            output = template.render(dynasty = self, path = locPath)
            if house:
                f = open(os.path.join(path, 'dynasties', dynid + '.html'), 'w', encoding='utf-8')
            else:
                f = open(os.path.join(path, 'dynasties', dynid + 'Dyn.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()

class gChar: #game character
    def __init__(self, charid:str, allData:str, env:Environment, path:str, limit:int) -> None:
        global knownCuls
        global knownChars
        global knownFaiths
        global knownTitles
        global knownDyns
        #a bunch of properties there both for dead and alive
        try:
            rawData = findCharData(charid, allData)
        except Exception as e:
            print('Finding character data of charid %s failed' % charid)
            print(e.with_traceback())
        knownChars[charid] = self
        self.charid = charid
        #name
        findName = re.findall(r'first_name="(.*?)"', rawData, re.S)
        self.name = gameStringToRead(findName[0].replace('_',''))
        #birth date
        findBirth = re.findall(r'birth=(.*?)\n', rawData, re.S)
        self.birth = findBirth[0]
        #culture
        findCulture = re.findall(r'culture=(.*?)\n', rawData, re.S)
        if len(findCulture) > 0:
            culid = findCulture[0]
            if culid in knownCuls.keys():
                self.culture = knownCuls[culid]
            else:
                self.culture = gCulture(culid, allData, env, path)                
        else:
            #uh no clue how but sometimes the culture is just.. missing from the savefile?
            self.culture = 'Lost to time...'
        findFaith = re.findall(r'faith=(.*?)\n', rawData, re.S)
        if len(findFaith) > 0:
            faid = findFaith[0]
            if faid in knownFaiths.keys():
                self.faith = knownFaiths[faid]
            else:
                self.faith = gFaith(faid, allData, env, path)
        else:
            self.faith = 'Lost to time...'
        #nickname
        try:
            findNick = re.findall(r'nickname="(.*?)"', rawData, re.S)
            self.nick = gameStringToRead(findNick[0])
        except:
            #boring mf
            self.nick = ''
            pass
        #dna
        findDna = re.findall(r'dna="(.*?)"', rawData, re.S)
        if len(findDna) > 0:
            self.dna = findDna[0]
        findSkills = re.findall(r'skill={(.*?)}', rawData, re.S)
        self.skills = findSkills[0].split(' ')
        findTraits = re.findall(r'traits={(.*?)}', rawData, re.S)
        traits = []
        if len(findTraits):
            findTraits = findTraits[0].split(' ')[1:-1]
            for trait in findTraits:
                traits.append(getTrait(int(trait)))
        self.traits = traits
        findRecessive = re.findall(r'recessive_traits={(.*?)}', rawData, re.S)
        self.recessive = []
        if len(findRecessive) > 0:
            for trait in findRecessive[0].split(' ')[1:-1]:
                self.recessive.append(getTrait(int(trait)))
        #family
        familyData = re.findall(r'family_data={(.*?)\t}', rawData, re.S)
        if len(familyData) > 0:
            familyData = familyData[0]
            findSpouse = re.findall(r'\tspouse=(.*?)\n', familyData, re.S)
            if len(findSpouse) > 0:
                self.spouses = findLinkedChars(findSpouse, limit, allData, env, path)
            findFormer = re.findall(r'former_spouses={(.*?)}', familyData, re.S)
            if len(findFormer) > 0:
                formerSpouses = findFormer[0].split(' ')[1:-1]
                self.former = findLinkedChars(formerSpouses, limit, allData, env, path)
            findChildren = re.findall(r'child={(.*?)}', familyData, re.S)
            if len(findChildren) > 0:
                children = findChildren[0].split(' ')[1:-1]
                self.children = findLinkedChars(children, limit, allData, env, path)
        #house is after the family so that all family members have been explored
        findDynasty = re.findall(r'dynasty_house=(.*?)\n', rawData, re.S)
        if len(findDynasty) > 0:
            self.dynastyId = findDynasty[0]
            if self.dynastyId in knownDyns.keys():
                self.house = knownDyns[self.dynastyId]
            else:
                self.house = gDynn(self.dynastyId, allData, env, path)
        else:
            self.house = 'Lowborn'
        if 'dead_data' in rawData:
            self.dead = True
            self.date = re.findall(r'date=(.*?)\n', rawData, re.S)[0]
            self.reason = gameStringToRead(re.findall(r'reason="(.*?)"\n', rawData, re.S)[0])
            findLiege = re.findall(r'liege=(.*?)\n', rawData, re.S)
            if len(findLiege) > 0:
                liege = findLiege[0]
                if(liege != charid):
                    self.liege = liege
            findGovernment = re.findall(r'government="(.*?)"', rawData, re.S)
            if len(findGovernment) > 0:
                self.government = findGovernment[0]
                findDomain = re.findall(r'domain={(.*?)}', rawData, re.S)
                titleList = findDomain[0].split(' ')[1:-1]
                self.titles = []
                for title in titleList:
                    if title in knownTitles.keys():
                        self.titles.append(knownTitles[title])
                    else:
                        self.titles.append(gTitle(title, allData, env, path))
            else:
                self.government = 'Unlanded'
        else:
            self.dead = False
            #the char isnt dead we need to parse other stuff
            findGold = re.findall(r'gold=(.*?)\n', rawData, re.S)
            self.gold = findGold[0]
            findPiety = re.findall(r'accumulated=(.*?)\n', rawData, re.S)
            self.piety = findPiety[0]
            self.prestige = findPiety[0]
            findKills = re.findall(r'kills={(.*?)}', rawData, re.S)
            if len(findKills) > 0 and limit > 0:
                killList = findKills[0].split(' ')[1:-1]
                self.kills = []
                for dead in killList:
                    if dead in knownChars.keys():
                        self.kills.append(knownChars[dead])
                    else:
                        self.kills.append(gChar(dead, allData, env, path, limit - 1))
            findLanguages = re.findall(r'languages={(.*?)}', rawData, re.S)
            if len(findLanguages) > 0:
                self.languages = []
                for lang in findLanguages:
                    self.languages.append(lang.replace('language_', ''))
            findGovernment = re.findall(r'government="(.*?)"', rawData, re.S)
            if len(findGovernment) > 0:
                self.government = findGovernment[0]
                findDomain = re.findall(r'domain={(.*?)}', rawData, re.S)
                titleList = findDomain[0].split(' ')[1:-1]
                self.titles = []
                for title in titleList:
                    if title in knownTitles.keys():
                        self.titles.append(knownTitles[title])
                    else:
                        self.titles.append(gTitle(title, allData, env, path))
                findVassals = re.findall(r'vassal_contracts={(.*?)}', rawData, re.S)
                if len(findVassals) > 0 and limit > 0:
                    self.vassals = []
                    for vassal in findVassals[0].split(' ')[1:-1]:
                        try:
                            vassalId = findVassal(vassal, allData)
                            if vassalId in knownChars.keys():
                                self.vassals.append(knownChars[vassalId])
                            else:
                                self.vassals.append(gChar(vassalId, allData, env, path, limit - 1))
                        except:
                            pass
                findDread = re.findall(r'dread=(.*?)\n', rawData, re.S)
                if len(findDread):
                    self.dread = findDread[0]
                else:
                    self.dread = 0
                findStrength = re.findall(r'current_strength=(.*?)\n', rawData, re.S)
                if len(findStrength) > 0:
                    self.strength = findStrength[0]
                else:
                    self.strength = 0
            else:
                self.government = 'Unlanded'
            findMemories = re.findall(r'memories={(.*?)}', rawData, re.S)[0].split(' ')[1:-1]
            if len(findMemories) > 0:
                self.memories = []
                for memory in findMemories:
                    self.memories.append(gMem(memory, allData))
        #save to global variable
        knownChars[charid] = self
        if env != False:
            #__file__
            absFilePath = os.path.realpath(__file__) 
            locPath = os.path.dirname(absFilePath) + '\\' + path 
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'charTemplate.html'))
            output = template.render(character = self, path = locPath)
            f = open(os.path.join(path, 'characters', charid + '.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()
        

class lChar: #lineage character
    def __init__(self, rawData:str, allData:str, env:Environment, path:str, limit:int):
        #always there
        findCharid = re.findall(r'character=(.*?)\n', rawData, re.S)
        self.charid = findCharid[0]
        #print(self.charid)
        findDate = re.findall(r'date=(.*?)\n', rawData, re.S)
        self.date = findDate[0]
        #we create a page for this character and save it to extract select elements
        charVar = gChar(self.charid, allData, env, path, limit)
        self.name = charVar.name
        self.nick = charVar.nick
        self.house = charVar.house
        #accomplishments
        try:
            findScore = re.findall(r'score=(.*?)\n', rawData, re.S)
            #print(findScore)
            self.score = findScore[0]
            findLifestyle = re.findall(r'lifestyle="(.*?)"', rawData, re.S)
            #print(findLifestyle)
            self.lifestyle = gameStringToRead(findLifestyle[0])
            findPerks = re.findall(r'perk="(.*?)"', rawData, re.S)
            #print(findPerks)
            self.perks = []
            for perk in findPerks:
                self.perks.append(gameStringToRead(perk))
        except:
            self.score = "This character's legacy is still undecided"
            self.lifestyle = ""
            self.perks = ""

class gLineage: #game lineage
    def __init__(self, rawData:str, allData:str, env:Environment, limit:int) -> None:
        findPlayer = re.findall(r'name="(.*?)"', rawData, re.S)
        self.player = findPlayer[0]
        print('Starting work on ' + self.player + ' history')
        path = self.player + ' history'
        #create directory
        try:
            os.makedirs(path)
        except:
            print('Directory is already here...')
        try:
            os.makedirs(os.path.join(path, 'characters'))
        except:
            print('Directory is already here...')
        try:
            os.makedirs(os.path.join(path, 'dynasties'))
        except:
            print('Directory is already here...')
        try:
            os.makedirs(os.path.join(path, 'cultures'))
        except:
            print('Directory is already here...')
        try:
            os.makedirs(os.path.join(path, 'faiths'))
        except:
            print('Directory is already here...')
        try:
            os.makedirs(os.path.join(path, 'titles'))
        except:
            print('Directory is already here...')
        #findLeadChar = re.findall(r'character=(.*?)\n', rawData, re.S)
        #self.lead = findLeadChar[0]
        #print(self.lead)
        legacyData = re.findall(r'legacy={(.*?)\n }', rawData, re.S)[0]
        #print(legacyData)
        charsData = re.findall(r'{(.*?)\t}', legacyData, re.S)
        #print(charsData)
        self.chars = []
        i = 1
        lngth = len(charsData)
        timeSince = time.time()
        sTs = []
        print('Starting character parsing')
        for charData in charsData:
            #print(charData)
            char = lChar(charData, allData, env, path, limit)
            sT = round(time.time() - timeSince, 2)
            sTs.append(sT)
            print(str(round(i/lngth * 100)) + '% done|s/it: ' + str(sT), flush=True, end='\r')
            timeSince = time.time()
            i = i + 1
            self.chars.append(char)
        print('Lineage mapping done|Avg s/it: ' + str(round(sum(sTs)/len(sTs),2)))
        #create home file
        #you can use False as env to override file creation
        if env != False:
            template = env.get_template(os.path.join(TEMPLATE_PATH, 'homeTemplate.html'))
            output = template.render(lineage = self)
            f = open(os.path.join(path, 'home.html'), 'w', encoding='utf-8')
            f.write(output)
            f.close()
            print('home.html created')

def findCharData(charid:str, data:str) -> str:
    charData = re.findall(r'\n%s={\n\tfirst_name=.+?\n}' % charid, data, re.S)
    return charData[0]

def findVassal(conId:str, data:str) -> str:
    conData = re.findall(r'\n\t\t%s={.+?\n\t\t}' % conId, data, re.S)[0]
    vassalId = re.findall(r'vassal=(.+?)\n', conData, re.S)[0]
    return vassalId

def findTitleData(titleid:str, data:str) -> str:
    titleData = re.findall(r'\n%s={.+?\n}' % titleid, data, re.S)
    return titleData[0]

def findMemData(memid:str, data:str) -> str:
    memData = re.findall(r'\t\t%s={\n\t\t\ttype=.+?\n\t\t}' % memid, data, re.S)
    return memData[0]

def getChrNameOrObj(charid:str, allData:str):
    global knownChars
    if charid in knownChars.keys():
        res = knownChars[charid]
    else:
        charData = findCharData(charid, allData)
        findName = re.findall(r'first_name="(.*?)"', charData, re.S)
        res = gameStringToRead(findName[0].replace('_',''))
    return res

def findLinkedChars(charids:list, limit:int, allData:str, env:Environment, path:str) -> list:
    global knownChars
    chars = []
    for char in charids:
        try:
            charData = findCharData(char, allData)
            #print(charData)
            if limit > 0:
                try:
                    if char in knownChars.keys():
                        chars.append(knownChars[char])
                    else:
                        chars.append(gChar(char, allData, env, path, limit-1))
                except:
                    findName = re.findall(r'first_name="(.*?)"', charData, re.S)
                    chars.append(gameStringToRead(findName[0].replace('_','')))
            else:
                findName = re.findall(r'first_name="(.*?)"', charData, re.S)
                chars.append(gameStringToRead(findName[0].replace('_','')))
        except:
            pass
        
    return chars

def getTrait(traitId:str) -> str:
    line = linecache.getline('trait_indexes.lookup', traitId + 1)
    return gameStringToRead(line)

def gameStringToRead(string:str) -> str:
    string = string.replace('dynn_', '').replace('_lifestyle', '').replace('_perk', '')
    string = string.replace('nick_', '').replace('death_', '').replace('ethos_', '')
    string = string.replace('heritage_', '').replace('martial_custom_', '').replace('tradition_', '')
    string = string.replace('fp2_', '').replace('fp1_', '').replace('language_', '')
    string = string.replace('_1', '').replace('_2', '').replace('doctrine_', '')
    string = string.replace('special_', '').replace('is_', '')
    #string = string.replace('A_', 'ã').replace('O_', 'õ').replace('E_', 'ẽ')
    string = string.replace('_',' ')
    string = string.lower().capitalize()
    return string

def findHouseData(dynid:str, data:str) -> str:
    i = 0
    houseData = re.findall(r'\n%s={\n\t.+?\n}' % dynid, data, re.S)
    while 'dynasty=' not in houseData[i]: #we have to iterate over all hits because pdx are not nice to me
        i = i + 1
    return houseData[i]

def findDynastyData(dynid:str, data:str) -> str:
    i = 0
    dynData = re.findall(r'\n%s={\n\t.+?\n}' % dynid, data, re.S)
    while 'prestige=' not in dynData[i]: #we have to iterate over all hits because pdx are not nice to me
        i = i + 1
    return dynData[i]

def getTitleName(titleid:str, allData:str) -> str:
    rawData = findTitleData(titleid, allData)
    findKey = re.findall(r'key="(.*?)"', rawData, re.S)
    findName = re.findall(r'name="(.*?)"', rawData, re.S)
    baseName = findName[0]
    if 'article' in rawData:
        findArt = re.findall(r'article="(.*?)"', rawData, re.S)
        return findArt[0] + baseName
    elif 'b_' in findKey[0]:
        return 'Barony of ' + baseName
    elif 'c_' in findKey[0]:
        return 'County of ' + baseName
    elif 'd_' in findKey[0]:
        return 'Duchy of ' + baseName
    elif 'k_' in findKey[0]:
        return 'Kingdom of ' + baseName
    elif 'e_' in findKey[0]:
        return 'Empire of ' + baseName
    else:
        return baseName

if __name__ == "__main__":
    #do the thing
    startTime = time.time()
    filename = input('Name of the readable ck3 save file:')
    sys.tracebacklimit = 10
    with open (filename + ".ck3", "r", encoding='utf-8') as myfile:
        data=myfile.read()
        print('File length: ' + str(len(data)) + ' characters, or: ' + str(len(data.split('\n'))) + ' lines')
        #we need to get a string containing only char data
        #ok but now we have to find the lineages in the save file
        charachterhistory = re.findall(r'played_character={.+?\n}', data, re.S)
        for lineageData in charachterhistory:
            try:
                lineage = gLineage(lineageData, data, Environment(loader=FileSystemLoader('')), limit=1)
            except Exception:
                print(traceback.format_exc())
        print('Done...')
        print("--- %s seconds ---" % (time.time() - startTime))
        input('Press any key to continue...')
