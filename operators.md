|Prefix Operator|Use|
|--:|:--|
|`+`|Identity function (must be a number)|
|`-`|Negative of target (must be a number)|
|`#`|Size function (length of string/tuple, cardinality of set)|
|`!`|Logical negation (boolean coersion)|
|`^`|Head|
|`$`|Last|
|`~`|Tail (equivalent to `1 @ [..]`)|
|`&`|Init (equivalent to `-1 @ [..]`)|
|`not`|Keyword form of prefix `!`|

### Infix Operators
Note: The keyword operators behave identically to their symbol operator counterparts **except for precedence**. The keyword forms all have a lower precedence than any symbol forms.

Examples:
- `!x && y` behaves like `(!x) && y`
- `not x && y` behaves like `!(x && y)`

|Infix Operator|Use|
|--:|:--|
|`<`/`>`/`<=`/`>=`|Compares numbers. If one or both of the operands is a collection, the expression treats them as their own size. eg `3 > [2..5]` is treated like `3 > 4` which is `false`, `[2..] < {2}|
|`==`/`!=`|Comparison of equality for 2 values. YSetL does not use reference equality, so two identical collections generated independently will be considered equal.|
|`+`|Addition of numbers; Union of sets; Concatenation of strings/tuples|
|`-`|Subtraction of numbers; Difference of sets|
|`*`|Multiplication of numbers; Intersection of sets; If used between an integer and a string/tuple, the collection is concatenated by itself an amount of times equal to the integer. If the number is zero, it returns the empty string/tuple.|
|`/`|Division of numbers, evaluates to integer if both values are integers, otherwise evaluates to float|
|`@`|Take operator. First operand must be an integer, and the second must be a tuple. The integer selects the index of the first element in the new tuple, eg `2 @ [10..100]` produces `[12, 13, ..., 99, 100]`. Negative numbers operate relative to the end, eg. `-2 @ [10..100]` produces `[10, 11, ..., 97, 98]`|
|`??`|Null coelescing|
|`**`|Exponentiation|
|`mod`|Modulus operation (both operands must be integers)|
|`&`|Bitwise AND|
|`\|`|Bitwise OR|
|`^`|Bitwise XOR|
|`<<`|If the first operand is a set, it returns a set with the second value inserted. If the first operand is a tuple, it returns a tuple with the second value pushed to the end. If both operands are integers, performs a bitwise left shift.|
|`>>`|If the first operand is a set, it returns a set with the second value removed. If the first operand is a tuple, it returns a typle with the second value pushed to the front. If both operands are integers, performs a bitwise right shift.|
|`&&`|Logical conjunction|
|`\|\|`|Logical disjunction|
|`%(expr)`/`%ident`|Reducer. Given the form `X %(expr) Y`, `Y` must evaluate to a collection. The `expr` must evaluate to a binary function. `X` is the initial accululator|
|`.(expr)`/`.ident`|Passes the 2 operands into the results of expr, as long as expr evaluates to a binary function. Equivalent to (expr)(X, Y)|
|`in`|Test for membership in a collection|
|`notin`|Negative form of infix `in`|
|`subset`|Test that the first operand is a subset of the second operand (both operands must be sets)|
|`impl`|Logical implication|
|`iff`|Logical equivalence (like infix `==`, but with different precendence)|
|`and`|Keyword form of infix `&&`|
|`or`|Keyword form of infix `\|\|`|
|`union`|Keyword form of infix `+`|
|`inter`|Keyword form of infix `*`|
|`div`|Keyword form of infix `/`|
|`with`|Keyword form of infix `<<`|
|`less`|Keyword form of infix `>>`|
