from collections import Counter
from pathlib import Path
from loguru import logger
import random
import time

from multirun.matching import MultiMatchingWithoutElo

# disable logging to speed things up
logger.disable("mjai");

# prolly can for loop over seed too if you want
seed = 69
# remove below line to get completely random
random.seed(seed);

# following can have more than 4, if say like 10 bots then for loop over matching.match
matching = MultiMatchingWithoutElo({
    1: Path("./examples/rulebase.zip"),
    #1: Path("./examples/tsumogiri.zip"),
    2: Path("./examples/tsumogiri.zip"),
    3: Path("./examples/tsumogiri.zip"),
    4: Path("./examples/tsumogiri.zip"),
}, nummatches = 100)

runname = f"lol_{seed}" # program this to take different values if you want multiple runs

# .match will pick 4 random players in random order, guranteed least played one is in it
try :
    starttime = time.time()
    detail = matching.match(runname) # save stuff to ./logs{runname}/{uuid}/
    endtime = time.time()
    print(f"time elapsed: {endtime - starttime}")
except (e):
    print(e)

matching.save_match_json(runname, detail) # optionally save to ./matching/{runname}.json

