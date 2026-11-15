def divmod2(a, b):
    return a // b, a % b

def main():
    t = (1, "x", 2.5)
    print(t, len(t), t[1], t[-1], (7,), ())
    q, r = divmod2(17, 5)
    print(q, r)
    xs = [10, 20, 30, 40, 50]
    print(xs[1:3], xs[:2], xs[3:], xs[::2], xs[::-1], xs[-2:], xs[1:-1])
    s = "hello world"
    print(s[0:5], s[6:], s[::-1], s[-3:])
    print(t[0:2], (1, 2) == (1, 2), (1, 2) in [(1, 2), (3, 4)])
    for a, b in [(1, 2), (3, 4)]:
        print(a + b)
    pairs = [(1, "a"), (2, "b")]
    for n, c in pairs:
        print(n, c)
    a, b = 1, 2
    a, b = b, a
    print(a, b)
