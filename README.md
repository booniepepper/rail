
![License](https://img.shields.io/github/license/booniepepper/rail)
![Lines of code](https://img.shields.io/tokei/lines/github/booniepepper/rail)
![GitHub repo size](https://img.shields.io/github/repo-size/booniepepper/rail)

# Rail

A straightforward programming language.

Rail is an experimental [concatenative](https://concatenative.org/wiki/view/Concatenative%20language)
virtual machine and minimal programming language. It is under wild development,
and currently zero stability between versions is guaranteed.

See also: [`dt`](https://github.com/booniepepper/dt) which is a looser language
with a similar syntax.

```
$ railsh
rail 0.30.2

> 1 1 + print
2

> [[n] -> n print " " print n 2 *] "print-and-double" def

> 1 [print-and-double] 7 times
1 2 4 8 16 32 64 

> [[false] ["bye"] [true] ["hi"]] ? println
hi
```

## Installation

```shell
$ cargo install rail-lang
$ railup bootstrap
```

## Credits

Available under GPL v2.

A side quest of J.R. Hill | https://so.dang.cool | https://github.com/booniepepper
