# Tuples
`x = [1...5];`  

### Ordered assignment
```
[a, b] = x;
```
- a => `1`
- b => `2`

### Spread-from-front assignment
```
[a, b, ...rest] = x;
```
- a = `1`
- b => `2`
- rest => `[3,4,5]`

```
[a, ~, ...rest] = x;
```
- a = `1`
- rest => `[3,4,5]`

### Spread-from-back assignment
```
[...rest, a, b] = x;
```
- rest => `[1,2,3]`
- a => `4`
- b => `5`

```
[...rest, ~, a] = x;
```
- rest => `[1,2,3]`
- a => `5`

### Spread-from-center assignment
```
[a, ...rest, b] = x;
```
- a => `1`
- rest => `[2,3,4]`
- b => `5`

```
[a, ...rest, ~] = x;
```
- a => `1`
- rest => `[2,3,4]`

# SetMaps
`x = { foo: 5, bar: 7, baz: 9 };`  

### Named assignment
```
{ foo } = x;
```
- foo => `5`

### Spread assignment
```
{ foo, ...rest } = x;
```
- foo => `5`
- rest => `{ bar: 7, baz: 9 }`
