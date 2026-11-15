def risky(n):
    if n == 0:
        raise ValueError("zero not allowed")
    return 10 // n

def main():
    for n in [2, 0]:
        try:
            print(risky(n))
        except ValueError as e:
            print("caught", e)
        finally:
            print("done", n)
    try:
        print(1 // 0)
    except ZeroDivisionError as e:
        print("div", e)
    try:
        xs = [1, 2]
        print(xs[5])
    except IndexError as e:
        print("idx", e)
    try:
        d = {"a": 1}
        print(d["b"])
    except KeyError as e:
        print("key", e)
    try:
        try:
            raise ValueError("inner")
        except ValueError as e:
            print("in", e)
            raise ValueError("outer")
    except ValueError as e:
        print("out", e)
    total = 0
    for i in range(5):
        try:
            if i % 2 == 0:
                raise ValueError("even")
            total += i
        except ValueError:
            continue
    print(total)
