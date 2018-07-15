#!/usr/bin/env python3

def get_weekday(day):
    return (day + 3) % 7

def upto(day, i):
    # return day
    return day + 6 - (day - i + 2) % 7;

print(end="day     │")
for day in range(50):
    print("{:2}│".format(day), end="")
print()

print(end="weekday │")
for day in range(50):
    print("\x1b[3{0}m{0:2}\x1b[m│".format(get_weekday(day)), end="")
print()

for uptoi in range(7):
    print(end="upto{}   │".format(uptoi))
    for day in range(50):
        print("{:2}│".format(upto(day, uptoi)), end="")
    print()
