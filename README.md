# Basic Prolog interpreter implemented in Rust
Newer and improved version inspired from the [_Prolog interpreter written in Haskell_](https://github.com/arctfx/prolog-interpreter-hs/tree/main)
_...with fewer bugs and fewer logical flaws_

## Въведение в задачата
Целта на проекта е да се напише на чист Rust език интерпретатор на опростена версия на логическия език за програмиране Prolog.
За нашите цели ще наричаме нашият език _SimpleProlog_.

_SimpleProlog_ има следната граматика:
```
identfier -> lowercase_letter[letter_or_number]
variable -> uppercase_letter[letter_or_numer]
constant -> identifier
term -> constant | variable | identifier(term[, term])
atom -> identifier(term[, term])
fact -> atom.
rule -> atom :- atom[, atom].
```
Дотук сме използвали само най-простите правила за синтаксис в _SimpleProlog_, като забелязваме, че нямаме аритметика и списъци. 
Специални оператори са `DotOperator` `.`, `ArrowOperator` `:-`, `LeftBracketOperator` `(`, `RightBracketOperator` `)`, `CommaOperator` `,`, `QueryOperator` `?-`.
В този контекст atom има повече смисъл като атомарна формула.

_SimpleProlog_ няма да бъде функциониращ без да имаме заявки. Затова ще добавим и следните граматични правила:
```
query -> ?- fact[, fact]
```
## To-Do
_Use Result instead of panic! in the tokenizer and parser._
## Structure

### UI
The user interface is pretty console using the `ratatui` crate.
It consists of 3 panes: Editor pane, Console pane and Output pane.

The Editor pane is used for editing the database text where the user can add rules and facts.

The Console pane is used for typing commands and queries.

The Output pane is used for writing the output of the commands and the results from the queries.

### Parser

### Abstract Syntax Tree / AST
AST is generated from tokenized statements.

### Intermediate Representation / IR
The main difference between the AST and IR is that terms can be either an atomic formula (a function with arguments, or a constant) or a variable.
Constants are technically functions with no arguments.

### Unification
For unification we are using the following algorithm scheme: 
```
Initialise the MGU to an empty unifier
Push T1 = T2 to the stack
While the stack is not empty
	Pop X = Y from the stack
	       case: X is a variable AND X does not occur in Y
        	Create unifier U such that X = Y
                Apply U to MGU
                Add U to MGU
                Apply U to stack
        case: Y is a variable AND Y does not occur in X
        	Create unifier U such that Y = X
                Apply U to MGU
                Add U to MGU
                Apply U to stack
        case: X and Y are identical constants or variables
        	do nothing
        case: X is of form p(a0,..,an) and Y is of form p(b0,..,bn)
		        For m = 0 to n
                	push am = bm to the stack
        default case:
        	Failure
Return the MGU
```
_Note: the algorithm is borrowed from the book The Art of Prolog._

### Solver
`resolve_query`

_To-Do: write documentation_