def main():
    xs = [3, 1, 2]
    xs.append(10)
    print(xs, len(xs), xs[0], xs[-1])
    xs[1] = 99
    xs[2] += 5
    print(xs, sum(xs))
    ys = sorted(xs)
    print(ys, xs)
    print(xs + [1, 2], 10 in xs, 7 not in xs)
    print(xs.pop(), xs.pop(0), xs)
    d = {"a": 1, "b": 2}
    d["c"] = 3
    d["a"] += 10
    print(d, len(d), "a" in d, "z" in d)
    print(d.get("b"), d.get("zz", -1), list(d.keys()), list(d.values()))
    total = 0
    for k, v in d.items():
        total += v
        print(k, v)
    print(total)
    for x in [1, 2, 3]:
        if x == 2:
            continue
        print(x)
    for ch in "abc":
        print(ch)
    words = "the quick brown fox".split()
    print(words, " ".join(words), "x-y-z".split("-"), "Hello".upper(), "  s ".strip())
    m = [[1, 2], [3, 4]]
    m[1][0] = 30
    print(m, m[1][0])
    print(["a", 'it\'s'], {1: "x"}, [1.5, True, None])
    evens = []
    for i in range(10):
        if i % 2 == 0:
            evens.append(i * i)
    print(evens)
