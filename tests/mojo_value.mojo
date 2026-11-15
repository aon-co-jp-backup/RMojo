trait Shape:
    fn area(self) -> Int: ...

@value
struct Rect(Shape, Stringable):
    var w: Int
    var h: Int

    fn area(self) -> Int:
        return self.w * self.h

    fn __str__(self) -> String:
        return "Rect(" + str(self.w) + "x" + str(self.h) + ")"

fn main():
    var a = Rect(2, 3)
    var b = a
    b.w = 10
    print(a, b, a.area(), b.area())
    var xs = List[Int](1, 2, 3)
    var ys = xs
    ys.append(4)
    print(xs, ys, len(xs), len(ys))
    var d = Dict[String, Int]()
    d["k"] = 5
    var e = d
    e["k"] = 6
    print(d, e)
    var v = SIMD[DType.float64, 4](1.0, 2.0, 3.0, 4.0)
    var w = v * 2.0 + v
    print(w, w.reduce_add(), w[2])
