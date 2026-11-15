def main():
    n = 3000000
    flags = []
    for i in range(n + 1):
        flags.append(True)
    i = 2
    while i * i <= n:
        if flags[i]:
            j = i * i
            while j <= n:
                flags[j] = False
                j += i
        i += 1
    c = 0
    for f in flags:
        if f:
            c += 1
    print(c - 2)
