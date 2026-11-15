struct Point:
    var x: Int
    var y: Int

    fn __init__(inout self, x: Int, y: Int):
        self.x = x
        self.y = y

    fn sum(self) -> Int:
        return self.x + self.y

    fn shift(inout self, d: Int):
        self.x += d
        self.y = self.y + d

struct Counter:
    var n: Int

    fn bump(inout self):
        self.n += 1

fn main():
    var p = Point(3, 4)
    print(p.sum())
    p.shift(10)
    print(p.x, p.y, p.sum())
    var c = Counter(0)
    for i in range(5):
        c.bump()
    print(c.n)
