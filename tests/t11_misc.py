w = 8
p = 2
x = 3.14159
print(f"[{x:{w}.{p}f}]")
print(f"[{42:>{w}}]")
a = {"a": 1, "b": 2}
b = {"b": 3, "c": 4}
d = {**a, **b, "z": 9}
print(len(d), d["b"], d["z"])
e = {"k": 0, **a}
print(len(e))
