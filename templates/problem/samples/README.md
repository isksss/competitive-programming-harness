# Samples

Store each sample as a matching pair:

```text
01.in
01.out
02.in
02.out
```

The harness compares output tokens line by line. Leading, trailing, and repeated
ASCII spaces or tabs within a line, LF versus CRLF, and one optional final
newline do not affect the result. Token order, line membership, and blank-line
positions must match, so moving a token to another line fails. This is a local
learning check that preserves output line structure.
