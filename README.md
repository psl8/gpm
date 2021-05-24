# GPM 

GPM was a macro language first designed in the mid 1960's by Christopher
Strachey. It was one of the first ever macro languages and would go on to
inspire Unix `m4` (of autotools fame) and to a lesser degree the C
preprocessor. This repo is a transliteration of the appendix of [the article
GPM was first described in](gpm) from CPL (yes, really) into Rust. The
comments, as well as the code's...interesting style have been largely retained.
GPM was intended to be used to implement a macroassembler that could be used to
bootstrap a CPL compiler. Since no CPL compiler existed at the time (or ever,
really) the code was written to be easy to hand-compile to machine code. As a 
result it uses a bunch of global variables effectively as machine registers, is
full of "GOTO soup" (here translated as method tail-calls), and uses a single
hardware stack to implement multiple logical stacks as linked lists through it.
The code is absolutely impenetrable.

You can run it like so (assuming Rust's `cargo` is installed):

```
$ cargo run < in.gpm > out.gpm
```

`in.gpm` is a nice little file of GPM examples taken from the article right
above their expected outputs. `out.gpm` should therefore have every line in it
twice right next to each other. (There is one example that doesn't work and
since it crashes the program I've quoted it to prevent evaluation. If you can
fix this I'd be happy to accept a PR, debugging this code is not easy
unfortunately). If you'd like you can also run the code interactively, it
listens on stdin and echos evaluated text back to stdout.  There is a command
`>` (a close quote on its own line) to end the session. A typical session might
look like:

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/gpm`
$DEF,foo,bar;

$foo;
bar
$DEF,twice,<~1~1>;

$twice,$foo;;
barbar
>
```

[gpm]: https://academic.oup.com/comjnl/article/8/3/225/336044
