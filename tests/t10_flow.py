a = {1, 2, 3}
b = {3, 4}
print(len(a | b), len(a & b), len(a ^ b), len(a - b))
print(6 | 1, 6 & 3, 6 ^ 3, 1 << 4, 256 >> 2)
def f(x):
    try:
        v = 10 // x
    except ZeroDivisionError:
        print("zero")
    else:
        print("ok", v)
    finally:
        print("fin")
f(2)
f(0)
def g():
    try:
        try:
            1 // 0
        except ZeroDivisionError:
            print("inner")
            raise
    except ZeroDivisionError:
        print("outer")
g()
