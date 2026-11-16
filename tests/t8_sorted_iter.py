def main():
    s = {3, 1, 2}
    print(sorted(s))
    print(sorted((5, 4, 6)))
    print(sorted({"b": 1, "a": 2}))
    print(sorted("hello"))
    print(sorted([3, 1, 2]))
    d = {i: i * i for i in range(4)}
    print(d)
    print(sorted({1, 2, 3} & {2, 3, 4}))
    i = 0
    while i < 2:
        i += 1
    else:
        print("wh-else", i)

main()
