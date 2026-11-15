def sieve(n):
    flags = [True] * 0 if False else []
    for i in range(n + 1):
        flags.append(True)
    flags[0] = False
    flags[1] = False
    i = 2
    while i * i <= n:
        if flags[i]:
            j = i * i
            while j <= n:
                flags[j] = False
                j += i
        i += 1
    count = 0
    for f in flags:
        if f:
            count += 1
    return count

def matmul(a, b, n):
    c = []
    for i in range(n):
        row = []
        for j in range(n):
            s = 0
            for k in range(n):
                s += a[i][k] * b[k][j]
            row.append(s)
        c.append(row)
    return c

def wordfreq(text):
    freq = {}
    for w in text.split():
        w = w.lower()
        if w in freq:
            freq[w] += 1
        else:
            freq[w] = 1
    return freq

def main():
    print(sieve(200000))
    n = 20
    a = [[i + j for j in range(n)] for i in range(n)] if False else []
    for i in range(n):
        r = []
        for j in range(n):
            r.append(i + j)
        a.append(r)
    c = matmul(a, a, n)
    print(c[3][4], c[19][19], sum(c[0]))
    f = wordfreq("the cat and the dog and the bird")
    print(f)
    print(sorted(list(f.keys())))
