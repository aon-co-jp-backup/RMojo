class AppError(Exception):
    pass


class NotFound(AppError):
    pass


def find(k):
    if k == 0:
        raise NotFound("missing")
    if k == 1:
        raise AppError("generic")
    return k


def run(k):
    try:
        print(find(k))
    except NotFound as e:
        print("notfound", e)
    except AppError as e:
        print("app", e)
    finally:
        print("done", k)


run(0)
run(1)
run(2)


def run2(k):
    try:
        find(k)
    except AppError as e:
        print("base caught", e)
    try:
        find(k)
    except Exception:
        print("exception caught")


run2(0)
run2(1)
