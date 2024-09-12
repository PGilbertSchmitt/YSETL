|Prefix Operator|Use|Done|
|--:|:--|:-:|
|`+`|Identity function (NO-OP)| ✔ |
|`-`|Negative of target (must be a number)| ✔ |
|`#`|Size function (length of string/tuple, cardinality of set)| ✔ |
|`!`|Logical negation (boolean coersion)| ✔ |
|`^`|Head| ✔ |
|`$`|Last| ✔ |
|`~`|Tail (equivalent to `1 @ [..]`)| ✔ |
|`&`|Init (equivalent to `-1 @ [..]`)| ✔ |
|`not`|Keyword form of prefix `!`| ✔ |

### Infix Operators
Note: The keyword operators behave identically to their symbol operator counterparts **except for precedence**. The keyword forms all have a lower precedence than any symbol forms.

Examples:
- `!x && y` behaves like `(!x) && y`
- `not x && y` behaves like `!(x && y)`
- `not x and y` behaves like `(!x) && y` (because `not` has higher preference than `and`)

|Infix Operator|Use|Done|
|--:|:--|:-:|
|`<` / `>` / `<=` / `>=`|Compares numbers. If one or both of the operands is a collection, the expression treats them as their own size. eg `3 > [2..5]` is treated like `3 > 4` which is `false`, `[2..] < {2}| ✔ |
|`==` / `!=`|Comparison of equality for 2 values. YSetL does not use reference equality, so two identical collections generated independently will be considered equal.| ✔ |
|`+`|Addition of numbers; Union of sets; Right merge of maps; Concatenation of strings/tuples| ✔ |
|`-`|Subtraction of numbers; Difference of sets| ✔ |
|`*`|Multiplication of numbers; Intersection of sets; Zip tuples; If used between an integer and a string/tuple, the collection is repeated an amount of times equal to the integer. If the number is zero, it returns the empty string/tuple.| ✔ |
|`/`|Division of numbers, evaluates to float if either values are floats, otherwise evaluates to int| ✔ |
|`?`|Modulus operation (both operands must be integers or floats)| ✔ |
|`@`|Take operator. First operand must be an integer, and the second must be a tuple. The integer selects the index of the first element in the new tuple, eg `2 @ [10..100]` produces `[12, 13, ..., 99, 100]`. Negative numbers operate relative to the end, eg. `-2 @ [10..100]` produces `[10, 11, ..., 97, 98]`| ✔ |
|`**`|Exponentiation. Evaluates to a float, unless both operands are integers AND the power is positive.| ✔ |
|`&`|Bitwise AND; Alternative set intersection/map merge| ✔ |
|`\|`|Bitwise OR; Alternative set union| ✔ |
|`^`|Bitwise XOR| ✔ |
|`<<`|If the first operand is a set, it returns a set with the second value inserted. If the first operand is a tuple, it returns a tuple with the second value pushed to the end. If both operands are integers, performs a bitwise left shift.| ✔ |
|`>>`|If the first operand is a set, it returns a set with the second value removed. If the first operand is a tuple, it returns a tuple with the second value pushed to the front. If the first operand is a map, it returns a map with the second operand (as a key) removed. If both operands are integers, performs a bitwise right shift.| ✔ |
|`&&`|Logical conjunction. This operator short circuits, so if the first operand evaluates to truthy, the second operand will not be evaluated.| ✔ |
|`\|\|`|Logical disjunction. This operator short circuits, so if the first operand evaluates to falsy, the second operand will not be evaluated.| ✔ |
|`??`|Null coelescing. This operator short circuits, so if the first operand is not null, the second operand will not be evaluated.| ✔ |
|`%ident` / `%(expr)` / `%binop` |Reduce operation. Given the form `X %(expr) Y`, `Y` must evaluate to a collection. The `expr` must evaluate to a binary function. For the form `X %binop Y`, the binop means any binary operator, but it must be a pure operator and not a compound operator (like Infix, Map insert, or another reduce). `X` is the initial accululator.| ✔ |
|`.ident` / `.(expr)`|Infix. Passes the 2 operands into the results of expr, as long as expr evaluates to a binary function. Equivalent to (expr)(X, Y)| ✔ |
|`<\|atom` / `<\|(expr)`|Map insert operation: First operand must be a map, `atom` or `expr` is the key, and the second operand is the value. Produces a new map with the key-value pair. The atom should not have the colon, so inserting value `V` into map `M` at symbol `:foo` would look like `M <|foo V` | |
|`in`|Test for membership in a collection| ✔ |
|`notin`|Negative form of infix `in`| ✔ |
|`subset`|Test that the first operand is a subset of the second operand (both operands must be sets)| ✔ |
|`impl`|Logical implication| ✔ |
|`iff`|Logical equivalence (like infix `==`, but with different precendence)| ✔ |
|`and`|Keyword form of infix `&&`| ✔ |
|`or`|Keyword form of infix `\|\|`| ✔ |
|`union`|Keyword form of infix `+`| ✔ |
|`inter`|Keyword form of infix `*`| ✔ |
|`div`|Keyword form of infix `/`| ✔ |
|`with`|Keyword form of infix `<<`| ✔ |
|`less`|Keyword form of infix `>>`| ✔ |
|`mod`|Keyword form of infix `?`| ✔ |
