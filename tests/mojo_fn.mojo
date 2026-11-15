fn fib(n: Int) -> Int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

fn main():
    var total: Int = 0
    for i in range(10):
        total += fib(i)
    print(total)
