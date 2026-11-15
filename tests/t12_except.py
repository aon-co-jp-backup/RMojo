def f(k):
    try:
        if k == 0:
            1 // 0
        elif k == 1:
            [1][5]
        else:
            {}["x"]
    except ZeroDivisionError:
        print("zero")
    except IndexError:
        print("index")
    except (KeyError, ValueError):
        print("key")
f(0)
f(1)
f(2)
try:
    try:
        1 // 0
    except KeyError:
        print("wrong")
except Exception:
    print("outer")
