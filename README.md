# Condensed Parser

Simple parsing exercise in rust that is able to parse a condensed sequence of characters and establish a relation as follow A -> B.

# Syntax
The parser support two type of syntaxes, a simple one and modern one.

### Rules
```cpp
// Simple syntax
"A,B"     = A -> B
"A,B,C,D" = A -> B, C -> D
// Modern syntax
"(A:B)"   = A -> B
"<A:B>"   = A -> B, B -> A
"(A:B,C)" = A -> B, A -> C
"<A:B,C>" = A -> B, A -> C, B -> A, C -> A
```
### Example
In simple syntax a code should always be followed by its pair
```cpp
"A, B, C, D" = A -> B, C -> D //OK
"A, B, C" // not ok C has no pair
```
We can mix modern syntax and old syntax
```cpp
"A, B, (C: D), E, F" -> A -> B, C -> D, E -> F
"A, B, <C: D>, E, F" -> A -> B, C -> D, D -> C, E -> F
```
The modern syntax can be useful in cases where simple syntax becomes repetitive therefore condensing the text.
```cpp
"A,B,A,C,A,D,A,E,A,F,A,G"         (23 chars) = "(A:B,C,D,E,F,G)" (15 chars)
"A,B,A,C,C,A,B,A,E,F,F,E,E,N,N,E" (31 chars) = "<A:B,C>,<E:F,N>" (15 chars)
```
# Bug or Feature?
There is currently one bug spotted, consider the following:
```cpp
"ABC,EFG(IJK:LMN)" //Invalid because of missing "," between EFG and '(' but still works with the current implementation.
```
This is should abort the parsing because of the missing "," but our implementation will still read this as if we have a "," in between.
This is a bug but it could be useful in case we really want to condense the text in less characters as much as possible by omitting those "," in these scenarios.
# Usage
Open command line and run:
```bash
$ rustc parser.rs
```
