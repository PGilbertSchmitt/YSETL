# YSETL Language

A small, set-based programming language based off of ISETL.

## History

In the beginning, there was [SETL](https://en.wikipedia.org/wiki/SETL). Showing up in 1969, it provided 2 composite data types: sets and tuples, and many built-in operations for working with sets. 2 decades later, Gary Levin, an associate professor of compsci at Clarkson University, developed ISETL (Interactive SETL) primary for use in 2 textbooks:
- **Learning Discrete Mathematics with ISETL** (1988, *ISBN 0-387-96898-9*)
- **Learning Abstract Algebra with ISETL** (1994, *ISBN 0-387-94152-5*)

3 decades after that, I went to Boston on my birthday and stopped at Brattle Book Shop, a very old used book store. My favorite section in used book stores is always the STEM section. Something about old math and programming books just hits different. On this particular day, I found the Abstract Algebra book listed above, in like-new condition with a floppy disk of the ISETL language still in an unopened envelope on the inside back cover. What a find. On my way home, I also found that someone actually put the ISETL source code on Github back in 2021, with a Make recipe to build on Ubuntu (and I would later realize that that _someone_ is Gary Levin himself). It definitely works, but I wanted something a little more modern: a smoother REPL experience, safer scoping rules for variables, better flow control, sleeker syntax, and some features that I would just like to have personally (like atom literals). The OG language has some interesting features that you don't normally see in modern languages (for better or worse), and while I want something that plays similarly to ISETL, I have omitted several features which I did not think were safe.

One of the major changes I've implemented is that all values are immutable. Operations that act on collections will generate new instances rather than modify them in-place. Is this a good idea? Probably not. Will I code it in such a way that it's highly performant? Not a chance. But is it worth it? Eh...

While my personal implementation isn't designed to be a hammer for every nail, I can absolutely see this being a simple alternative to ISETL for use in the above textbooks (which is the whole reason I started this adventure). If this is in any usable state by the end of the year, I may try to tackle Advent of Code at the end of [CURRENT YEAR] in YSETL.

## Name

There's nothing special about the name **YSETL**, and I'm not breaking any new ground here. I just wanted something with **-SETL** in the name, and "Y" is funny because truly, I have to ask myself: _"y r u doin this??"_. The answer, unsurprisingly, is `¯\_( ͡° ͜ʖ ͡°)_/¯`

---

## Features

### DataTypes:
- [x] Booleans
- [x] Integers
- [x] Floats
- [x] Strings
- [x] Atoms
- [ ] Tuples (Lists)
- [ ] Sets
- [ ] Maps (specialized Sets)
- [ ] Functions
- [ ] Function Maps (specialized Maps)

### Operations
- [ ] Arithmetic
- [ ] Control flow
- [ ] Global variables
- [ ] Local variables
- [ ] Boolean operations
- [ ] Tuple operations
- [ ] Set operations
- [ ] Map operations
- [ ] Iteration

### Other
- [ ] REPL
- [ ] IO
- [ ] Separate Compilation and Execute steps (aka running prebuilt binaries)
