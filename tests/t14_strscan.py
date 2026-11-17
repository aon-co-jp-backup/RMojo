def main():
    s = "ab&cd=" * 40 + "あい&う" + "x" * 130 + "&"
    print(len(s), s.count("&"), s.count("="), s.count("あ"), s.count("&c"), s.count("zz"), s.count(""))
    print(s.find("&"), s.find("="), s.find("う"), s.find("zz"), s.find("x"), s.find("cd="))
    t = "x" * 300 + "!" + "y" * 5
    print(t.find("!"), t.count("!"), t.count("y"), t.find("?"))
    u = "日本語" * 50 + "." + "end"
    print(u.find("."), u.count("日"), u.count("本語"))
    print("".count("a"), "".find("a"), "aaa".count("aa"))
