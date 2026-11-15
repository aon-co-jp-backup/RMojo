s = {3, 1, 2, 3, 1}
print(len(s))
print(2 in s, 9 in s)
t = {x % 3 for x in range(10)}
print(len(t))
s.add(7)
s.add(7)
print(len(s))
s.remove(7)
s.discard(100)
print(len(s))
u = set([1, 2, 2, 5])
print(len(u))
tot = 0
for x in u:
    tot += x
print(tot)
print(len(s.union(u)), len(s.intersection(u)), len(s.difference(u)))
e = set()
print(e)
print(sorted(list(s)))
print(len({c for c in "hello"}))
