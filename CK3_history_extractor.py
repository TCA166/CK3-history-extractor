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

#(.*?) - narrow search
#.+? - wide search

class gTitle:
    def __init__(self, titleid, allData, env, path):
        self.titleid = titleid
        #print(titleid)
        rawData = findTitleData(titleid, allData)
        findKey = re.findall(r'key="(.*?)"', rawData, re.S)
        self.key = findKey[0]
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        baseName = findName[0]
        if 'b_' in self.key:
            self.name = 'Barony of ' + baseName
        elif 'c_' in self.key:
            self.name = 'County of ' + baseName
        elif 'd_' in self.key:
            self.name = 'Duchy of ' + baseName
        elif 'k_' in self.key:
            self.name = 'Kingdom of ' + baseName
        elif 'e_' in self.key:
            self.name = 'Empire of ' + baseName
        self.baseName = baseName
        if 'history=' in rawData:
            pass #maybe one day i will manage to come up with a way to parse that mess
        try:
            global knownTitles
            knownTitles.update({titleid:self})
        except:
            pass

class gFaith:
    def __init__(self, faid, allData, env, path):
        self.faid = faid
        faiData = re.findall(r'\n\tfaiths={(.*?)\n}', allData, re.S)[0]
        rawData = re.findall(r'\n\t\t%s={(.*?)\n\t\t}' % faid, faiData, re.S)[0]
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        if len(findName) > 0:
            self.name = findName[0].capitalize()
        else:
            findTag = re.findall(r'tag="(.*?)"', rawData, re.S)
            self.name = gameStringToRead(findTag[0]).capitalize()
        #print(self.name)
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
            #print(doctrine)
            #print(gameStringToRead(doctrine))
            doctrines.append(gameStringToRead(doctrine))
        #print(doctrines)
        self.doctrines = doctrines
        findReligion = re.findall(r'religion=(.*?)\n', rawData, re.S)
        self.religion = findReligion[0]
        findHead =  re.findall(r'religious_head=(.*?)\n', rawData, re.S)
        #print(findHead)
        if len(findHead) > 0:
            #print(findHead[0])
            try:
                global knownTitles
                self.head = knownTitles[findHead[0]]
                #print('found')
            except KeyError:
                #print('not found')
                try:
                    self.head = gTitle(findHead[0], allData, env, path)
                except:
                    pass
                #print(self.head.name)
        try:
            global knownFaiths
            knownFaiths.update({faid:self})
        except:
            pass
        if env != False:
            template = env.get_template('faithTemplate.html')
            output = template.render(faith = self)
            #print(output)
            #print(path + '\\faiths\\' + faid + '.html')
            #print(os.path.isdir(path + '\\faiths\\'))
            f = open(path + '\\faiths\\' + faid + '.html', 'w')
            f.write(output)
            f.close()

class gCulture:
    def __init__(self, culid, allData, env, path):
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
                try:
                    global knownCuls
                    self.parents.append(knownCuls[parent])
                except KeyError:
                    #print(parent)
                    self.parents.append(gCulture(parent, allData, env, path))
        findTraditions = re.findall(r'traditions={(.*?)}', rawData, re.S)
        traditions = findTraditions[0].split(' ')[1:-1]
        self.traditions = []
        for tradition in traditions:
            self.traditions.append(gameStringToRead(tradition))
        findMartial = re.findall(r'martial_custom="(.*?)"', rawData, re.S)
        self.martial = gameStringToRead(findMartial[0])
        try:
            knownCuls.update({culid:self})
            #print(knownCuls[culid].name)
        except:
            pass
        if env != False:
            template = env.get_template('cultureTemplate.html')
            output = template.render(culture = self)
            f = open(path + '\cultures\\' + culid + '.html', 'w')
            f.write(output)
            f.close()



class gDynn:
    def __init__(self, dynid, allData, env, path, allChars):
        rawData = findDynastyData(dynid, allData)
        findName = re.findall(r'name="(.*?)"', rawData, re.S)
        try:
            self.name = gameStringToRead(findName[0])
        except:
            #print(rawData)
            #print(dynid)
            findName = re.findall(r'key="(.*?)"', rawData, re.S)
            self.name = gameStringToRead(findName[0].replace('dynn_','').replace('_',''))
        #print(self.name)
        self.dynid = dynid
        findDate = re.findall(r'found_date=(.*?)\n', rawData, re.S)
        self.date = findDate[0]
        try:
            findParent = re.findall(r'\tdynasty=(.*?)\n', rawData, re.S)
            parentId = findParent[0]
            if parentId != dynid:
                try:
                    global knownDyns
                    #print(knownDyns[parentId].name)
                    self.parent = knownDyns[parentId]
                except KeyError:
                    self.parent = gDynn(parentId, allData, env, path, allChars)
        except:
            pass
        self.members = allData.count('dynasty_house=' + dynid)
        findHistorical = re.findall(r'historical={(.*?)}', rawData, re.S)
        historicalLeaders = findHistorical[0].split(' ')[1:-1]
        #print(historicalLeaders)
        leaders = []
        for leader in historicalLeaders:
            try:
                rawData = findCharData(leader, allChars)
                findName = re.findall(r'first_name="(.*?)"', rawData, re.S)
                #print(rawData)
                leaders.append({'name':gameStringToRead(findName[0].replace('_','')),'charid':leader})
            except:
                pass
        self.leaders = leaders
        try:
            knownDyns.update({dynid:self})
            #print(knownDyns[dynid].name)
        except:
            pass
        if env != False:
            template = env.get_template('dynastyTemplate.html')
            absFilePath = os.path.realpath(__file__) 
            locPath = os.path.dirname(absFilePath) + '\\' + path 
            output = template.render(dynasty = self, path = locPath)
            f = open(path + '\dynasties\\' + dynid + '.html', 'w')
            f.write(output)
            f.close()

class gChar: #game character
    def __init__(self, charid, allData, allChars, env, path, limit):
        #a bunch of properties there both for dead and alive
        try:
            rawData = findCharData(charid, allChars)
        except:
            return
        self.charid = charid
        #name
        findName = re.findall(r'first_name="(.*?)"', rawData, re.S)
        self.name = gameStringToRead(findName[0].replace('_',''))
        #print(self.name)
        #birth date
        findBirth = re.findall(r'birth=(.*?)\n', rawData, re.S)
        self.birth = findBirth[0]
        #culture
        findCulture = re.findall(r'culture=(.*?)\n', rawData, re.S)
        if len(findCulture) > 0:
            culid = findCulture[0]
            try:
                global knownCuls
                self.culture = knownCuls[culid]
            except KeyError:
                #print(culid)
                self.culture = gCulture(culid, allData, env, path)
        else:
            #uh no clue how but sometimes the culture is just.. missing from the savefile?
            self.culture = 'Lost to time...'
        findFaith = re.findall(r'faith=(.*?)\n', rawData, re.S)
        if len(findFaith) > 0:
            faid = findFaith[0]
            try:
                global knownFaiths
                self.faith = knownFaiths[faid]
            except KeyError:
                self.faith = gFaith(faid, allData, env, path)
        else:
            self.faith = 'Lost to time...'
        #nickname
        try:
            findNick = re.findall(r'nickname="(.*?)"', rawData, re.S)
            #print(findNick)
            self.nick = gameStringToRead(findNick[0])
        except:
            #boring mf
            self.nick = ''
            pass
        #dna
        findDna = re.findall(r'dna="(.*?)"', rawData, re.S)
        if len(findDna) > 0:
            self.dna = findDna[0]
        findDynasty = re.findall(r'dynasty_house=(.*?)\n', rawData, re.S)
        if len(findDynasty) > 0:
            self.dynastyId = findDynasty[0]
            #print(dynastyData)
            try:
                global knownDyns
                #print(knownChars[self.dynastyId].name)
                self.house = knownDyns[self.dynastyId]
            except KeyError:
                self.house = gDynn(self.dynastyId, allData, env, path, allChars)
        else:
            self.house = 'Lowborn'
        findSkills = re.findall(r'skill={(.*?)}', rawData, re.S)
        self.skills = findSkills[0].split(' ')
        findTraits = re.findall(r'traits={(.*?)}', rawData, re.S)
        traits = []
        if len(findTraits):
            findTraits = findTraits[0].split(' ')[1:-1]
            
            for trait in findTraits:
                #print(trait)
                traits.append(getTrait(int(trait)))
        self.traits = traits
        findRecessive = re.findall(r'recessive_traits={(.*?)}', rawData, re.S)
        self.recessive = []
        if len(findRecessive) > 0:
            for trait in findRecessive[0].split(' ')[1:-1]:
                self.recessive.append(getTrait(int(trait)))
        #family
        familyData = re.findall(r'family_data={(.*?)\t\t}', rawData, re.S)
        if len(familyData) > 0:
            familyData = familyData[0]
            findSpouse = re.findall(r'\tspouse=(.*?)\n', familyData, re.S)
            if len(findSpouse) > 0:
                self.spouses = findLinkedChars(findSpouse, allChars, limit, allData, env, path)
            findFormer = re.findall(r'former_spouses={(.*?)}', familyData, re.S)
            if len(findFormer) > 0:
                formerSpouses = findFormer[0].split(' ')[1:-1]
                self.former = findLinkedChars(formerSpouses, allChars, limit, allData, env, path)
            findChildren = re.findall(r'child={(.*?)}', familyData, re.S)
            if len(findChildren) > 0:
                children = findChildren[0].split(' ')[1:-1]
                self.children = findLinkedChars(children, allChars, limit, allData, env, path)
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
            #print(findGovernment)
            if len(findGovernment) > 0:
                self.government = findGovernment[0]
                findDomain = re.findall(r'domain={(.*?)}', rawData, re.S)
                titleList = findDomain[0].split(' ')[1:-1]
                #print(titleList)
                self.titles = []
                for title in titleList:
                    try:
                        global knownTitles
                        self.titles.append(knownTitles[title])
                    except KeyError:
                        #print(title)
                        self.titles.append(gTitle(title, allData, env, path))
            else:
                self.government = 'Unlanded'
        else:
            #print(self.charid)
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
                #print(killList)
                self.kills = []
                for dead in killList:
                    #print(dead)
                    try:
                        global knownChars
                        self.kills.append(knownChars[dead])
                    except KeyError:
                        self.kills.append(gChar(dead, allData, allChars, env, path, limit - 1))
            findGovernment = re.findall(r'government="(.*?)"', rawData, re.S)
            if len(findGovernment) > 0:
                self.government = findGovernment[0]
                #print('reached')
                findDomain = re.findall(r'domain={(.*?)}', rawData, re.S)
                titleList = findDomain[0].split(' ')[1:-1]
                self.titles = []
                for title in titleList:
                    try:
                        self.titles.append(knownTitles[title])
                    except KeyError: 
                        self.titles.append(gTitle(title, allData, env, path))
                findVassals = re.findall(r'vassal_contracts={(.*?)}', rawData, re.S)
                if len(findVassals) > 0 and limit > 0:
                    self.vassals = []
                    for vassal in findVassals[0].split(' ')[1:-1]:
                        try:
                            vassalId = findVassal(vassal, allData)
                            try:
                                self.vassals.append(knownChars[vassalId])
                            except KeyError:
                                #print(vassalId)
                                self.vassals.append(gChar(vassalId, allData, allChars, env, path, limit - 1))
                        except:
                            #print(vassal)
                            #print(traceback.format_exc())
                            pass
                #print('vassals done')
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

        #save to global variable
        try:
            knownChars.update({charid:self})
        except:
            pass
        if env != False:
            #__file__
            absFilePath = os.path.realpath(__file__) 
            locPath = os.path.dirname(absFilePath) + '\\' + path 
            template = env.get_template('charTemplate.html')
            output = template.render(character = self, path = locPath)
            f = open(path + '\characters\\' + charid + '.html', 'w', encoding='utf-8')
            f.write(output)
            f.close()
        

class lChar: #lineage character
    def __init__(self, rawData, allChars, allData, env, path, limit):
        #print(rawData)
        #always there
        findCharid = re.findall(r'character=(.*?)\n', rawData, re.S)
        self.charid = findCharid[0]
        #print(self.charid)
        findDate = re.findall(r'date=(.*?)\n', rawData, re.S)
        self.date = findDate[0]
        #print(charData)
        #we create a page for this character and save it to extract select elements
        charVar = gChar(self.charid, allData, allChars, env, path, limit)
        self.name = charVar.name
        self.nick = charVar.nick
        self.house = charVar.house
        #print(self.name)
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
    def __init__(self, rawData, allChars, allData, env, limit):
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
            os.makedirs(path + '\characters')
        except:
            print('Directory is already here...')
        try:
            os.makedirs(path + '\dynasties')
        except:
            print('Directory is already here...')
        try:
            os.makedirs(path + '\cultures')
        except:
            print('Directory is already here...')
        try:
            os.makedirs(path + '\\faiths')
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
            char = lChar(charData, allChars, allData, env, path, limit)
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
            template = env.get_template('homeTemplate.html')
            output = template.render(lineage = self)
            f = open(path + '\\home.html', 'w')
            f.write(output)
            f.close()
            print('home.html created')

def findCharData(charid, data):
    charData = re.findall(r'\n\t%s={.+?\n\t}' % charid, data, re.S)
    return charData[0]

def findVassal(conId, data):
    conData = re.findall(r'\n\t\t%s={.+?\n\t\t}' % conId, data, re.S)[0]
    vassalId = re.findall(r'vassal=(.+?)\n', conData, re.S)[0]
    return vassalId

def findTitleData(titleid, data):
    titleData = re.findall(r'\n%s={.+?\n}' % titleid, data, re.S)
    return titleData[0]

def findLinkedChars(charids, allChars, limit, allData, env, path):
    chars = []
    for char in charids:
        try:
            charData = findCharData(char, allChars)
            #print(charData)
            if limit > 0:
                try:
                    try:
                        global knownChars
                        chars.append(knownChars[char])
                    except KeyError:
                        chars.append(gChar(char, allData, allChars, env, path, limit-1))
                except:
                    findName = re.findall(r'first_name="(.*?)"', charData, re.S)
                    chars.append(gameStringToRead(findName[0].replace('_','')))
            else:
                findName = re.findall(r'first_name="(.*?)"', charData, re.S)
                chars.append(gameStringToRead(findName[0].replace('_','')))
        except:
            pass
        
    return chars

def getAllChars(allData):
    living = re.findall(r'living={.+?\n}\n', allData, re.S)[0]
    dead = re.findall(r'dead_unprunable={.+?\n}\n', allData, re.S)[0]
    characters = re.findall(r'characters={.+?\n}\n', allData, re.S)[0]
    #for whatever reason pdx stores chars in 3 DIFFERENT ARRAYS WTF
    allChars = living + dead + characters
    return allChars

def getTrait(traitId):
    line = linecache.getline('trait_indexes.lookup', traitId + 1)
    return gameStringToRead(line)

def gameStringToRead(string):
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

def findDynastyData(dynid, data):
    i = 0
    dynData = re.findall(r'\n%s={.+?\n}' % dynid, data, re.S)
    while 'found_date' not in dynData[i]: #we have to iterate over all hits because pdx are not nice to me
        i = i + 1
    return dynData[i]

if __name__ == "__main__":
    #do the thing
    startTime = time.time()
    filename = input('Name of the readable ck3 save file:')
    sys.tracebacklimit = 10
    with open (filename + ".ck3", "r", encoding='utf-8') as myfile:
        data=myfile.read()
        print('File length: ' + str(len(data)) + ' characters, or: ' + str(len(data.split('\n'))) + ' lines')
        #we need to get a string containing only char data
        allChars = getAllChars(data)
        #ok but now we have to find the lineages in the save file
        charachterhistory = re.findall(r'played_character={.+?\n}', data, re.S)
        for lineageData in charachterhistory:
            try:
                #print(lineageData)
                lineage = gLineage(lineageData, allChars, data, Environment(loader=FileSystemLoader('')), 1)
            except Exception:
                print(traceback.format_exc())
        print('Done...')
        print("--- %s seconds ---" % (time.time() - startTime))
        input('Press any key to continue...')
