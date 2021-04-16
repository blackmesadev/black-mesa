# Argument Types

In command documentation, you've probably seen one of these:

-   `<reason:string...>`
-   `<target:user[]>`
-   `[time:duration]`

These are argument types. They are a way for users to identify what type is required for a certain
argument when giving them a value. These are purely a documentation visualisation rather than
something used by the bot itself.

As a quick breakdown, this is what each part means (in this case, `<reason:string...>`):

-   `<` and `>` are the [requirements](#argument-requirements).
-   `reason` is the name of the argument.
-   `:` is a separator between the name and the type.
-   `string` is the type of the argument.
-   `...` is a type modifier.

## Type Aliases

For simplicity, some parts of the documentation use a simplified type. The aliases are listed here.

| Alias  | Full Type                           |
| ------ | ----------------------------------- |
| `user` | <code>mention&#124;snowflake</code> |

## Argument Requirements

Arguments are surrounded by either `[]` or `<>`. These show the requirement level of the argument.
Optional arguments are surrounded by `[]` and required arguments are surrounded by `<>`.

## Type Modifiers

Type modifiers change the way a type is interpreted, including taking multiple of the same type as
one argument.

### Array Types (`[]`)

Array types are a modifier that show that an argument can take multiple of the same type.

### Greedy Types (`...`)

Greedy types are a modifier that is only used at the end of an argument list and is always used
with strings that take the rest of the input and turn it into one argument.

### Union Types (`|`)

Union types are a modifier that joins two different types together.
