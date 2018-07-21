import re
from subprocess import call

f = open("src/db.rs")

mode = 0
text = None

r = re.compile(r"""
    \((:user_id\ =\ 0)\ OR\ (:user_id\ =\ [a-z_.]+)\)
""", re.X)

#import sqlite3
#db = sqlite3.connect("tmp.db")

call(["rm", "-f", "tmp.db"])

def check(text):
    m = min((len(x) - len(x.lstrip()) for x in text if len(x.strip())))
    text = "\n".join([x[m:] for x in text]).split(";")
    text = filter(lambda x: len(x.strip()), text)
    text = map(lambda x: x.strip("\n"), text)
    text = list(text)
    for text in text:
        if text.startswith("CREATE "):
            call(["sqlite3", "tmp.db", text])
        elif text.startswith("SELECT ") or text.startswith("UPDATE "):

            m = r.search(text)
            if m:
                texts = [r.sub(r"1", text), r.sub(r"\2", text)]
            else:
                texts = [text]

            for text in texts:
                print("\x1b[33m{}\x1b[m".format(text))
                text1 = text.replace("(:user_id = 0 OR ", "(")
                call(["sqlite3", "tmp.db", "EXPLAIN QUERY PLAN\n" + text1])
            print("")

        elif text.startswith("INSERT "):
            pass
        else:
            print("\x1b[31m{}\x1b[m".format(text))

for line in f:
    if line.rstrip().endswith('"'):
        if text is None:
            text = []
        else:
            print("Unexpected")
            exit(1)
    elif line.lstrip().startswith('"'):
        if text is None:
            print("Unexpected")
            exit(1)
        else:
            check(text)
            text = None
    else:
        if text is not None:
            text.append(line.rstrip("\n"))
