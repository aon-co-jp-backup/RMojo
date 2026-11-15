from collections import List
from math import sqrt

struct Pair[T: Copyable]:
    var a: T
    var b: T

    fn __init__(inout self, a: T, b: T):
        self.a = a
        self.b = b

    fn swap(self) -> Pair[T]:
        return Pair[T](self.b, self.a)

fn first[T: Copyable](xs: List[T]) -> T:
    return xs[0]

fn main():
    var p = Pair[Int](1, 2)
    var q = p.swap()
    print(q.a, q.b)
    print(first[Int](List[Int](7, 8, 9)))
    print(sqrt(16.0))
