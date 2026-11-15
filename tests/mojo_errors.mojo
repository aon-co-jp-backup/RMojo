fn risky(n: Int) raises -> Int:
    if n == 0:
        raise Error("zero")
    return 10 // n

fn main():
    try:
        print(risky(5))
        print(risky(0))
    except e:
        print("caught", e)
