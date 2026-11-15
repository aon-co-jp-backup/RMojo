d = {k: k*2 for k in range(3)}
print(d)
i = 0
while i < 3:
    i += 1
else:
    print("else", i)
for j in range(3):
    if j == 5:
        break
else:
    print("for-else")
for j in range(3):
    if j == 1:
        break
else:
    print("NO")
x = 5
print(f"x={x} {x*2:>4} {3.14159:.2f} {'ab':^6}|{7:03d}")
