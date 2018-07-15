#!/usr/bin/env python3

import json
import urllib.request

def get_weekday(day):
    return (day + 3) % 7

def get(chat, args):
    url = 'http://127.0.0.1:3000/stats/' + chat + "?"
    url += "&".join(map(lambda x:"{}={}".format(*x), args.items()))
    with urllib.request.urlopen(url) as response:
        return json.loads(response.read())

class Test:
    def __init__(self):
        pass

    def begin(self, args):
        self.args = args
        self.failed = False

    def test(self, a, op, b, text):
        fail = \
                (op == "!=" and not (a != b)) or \
                (op == "==" and not (a == b)) or \
                (op == "<"  and not (a <  b)) or \
                (op == ">"  and not (a >  b)) or \
                (op == ">=" and not (a >= b)) or \
                (op == "<=" and not (a <= b)) or \
                False

        if fail:
            if not self.failed:
                self.failed = True
                print()
                print("Args: {}".format(self.args))
            print("  Fail: {}".format(text))
            print("    Test: {} {} {}".format(a, op, b))

t = Test()

v_all = get("@caninas", {"offset":7})

for weekday in range(7):
    for day in range(17592, 17630):
        args = {"offset":7, "from":day, "to":17630, "weekday":weekday}
        t.begin(args)
        v = get("@caninas", args)

        t.test(get_weekday(v["start_day"]), '==', weekday,
             'invalid weekday')

        t.test(v["start_day"], '>=', day,
               'start_day after day')

        t.test(v["start_day"] - day, '<', 7,
               'too big skip')

        days = map(lambda n: v["start_day"] + n*v["skip_day"], range(len(v["daily_users"])))
        for t_day, t_v0, t_v1 in zip(days, v["daily_users"], v["daily_messages"]):
            if v_all["daily_users"][t_day - v_all["start_day"]] != t_v0:
                print("failed: t_v0")
            if v_all["daily_messages"][t_day - v_all["start_day"]] != t_v1:
                print("failed: t_v1")
