# Motivation

Normalization attempts to transform an expression into a minimal sequence of
_non-overlapping_, normal rules. The goal is _not_ to make the expression
shorter but instead to make as readable as possible. For example, the
additional operator `,` is less known and can be mistaken with any other kind
of sequence (eg. in a day selector `Mo,Fr`).

Normalization is [_idempotent_][wiki-idempotence], which means that normalizing
an already normalized expression won't change the result.

## Examples

| input                                          | normalized                                                                  |
| ---------------------------------------------- | --------------------------------------------------------------------------- |
| `Mo-Su 00:00-24:00`                            | `24/7`                                                                      |
| `24/7 ; Su closed`                             | `Mo-Sa`                                                                     |
| `Mo-Su 10:00-12:00, Mo-Fr 14:00-18:00`         | `Mo-Fr 10:00-12:00,14:00-18:00; Sa-Su 10:00-12:00`                          |
| `10:00-18:00; Jul-Aug 10:00-22:00`             | `Jan-Jun,Sep-Dec 10:00-18:00; Jul-Aug 10:00-22:00`                          |
| `Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00` | `Mo-Fr 10:00-18:00; Jan-Jun,Sep-Dec Su 10:00-18:00; Jul-Aug Su 10:00-22:00` |

## Unsupported syntax

Not all syntax can be normalized, but this library will still do some best
effort by normalizing the longest prefix possible and keeping all rules after
the first unsupported one unchanged.

Here is an exhausting list of the kind of syntax you can't expect to see
normalized by current implementation:

| kind                                                    | behavior                   | example (1)                  |
| ------------------------------------------------------- | -------------------------- | ---------------------------- |
| [fallback rule][spec-fallback]                          | stop normalization (2)     | `Mo-Fr \|\| unknown`         |
| any range with steps                                    | stop normalization (2)     | `2000-3000/5`                |
| [monthday range][spec-monthday-range] with fixed dates  | stop normalization (2)     | `Mar31-Jun01`                |
| [monthday range][spec-monthday-range] with year         | stop normalization (2)     | `2025Jun-Aug`                |
| [weekday range][spec-weekday-range] with index in month | stop normalization (2)     | `Mo[2]`, `Mo[2] +1 days`     |
| [weekday range][spec-weekday-range] with a holiday      | stop normalization (2)     | `easter`                     |
| time that overlaps with next day                        | stop normalization (2)     | `22:00-06:00`, `22:00-28:00` |
| time with a solar event                                 | no time simplification (3) | `sunrise-18:00`              |
| time with an open end                                   | no time simplification (3) | `12:00-16:00+`               |
| time with repetition                                    | no time simplification (3) | `12:00-16:00/02:00`          |

Notes :

1. All the examples above contain a single rule, so they would be left
   unchanged by the normalization.
2. This rule and any following rule won't be treated.
3. This won't halt normalization but the algorithm won't try to merge this time
   range with others.

If a feature is not implemented I may have considered it to be too niche for
the effort. Feel free to [open an issue][gh-issues] on Github or open a merge
request if you disagree!

# How it works

## Build a canonical time table

First, create a "canonical" time table over 4 dimensions (year, month, weeknum,
daynum), each cell keeps track of time ranges recorded for a single combination
of intervals over those 4 dimensions. Cells are always non-overlapping and can
be split while processing the expression if necessary.

For example, the resulting structure looks like this (simplified to 2
dimensions for obvious reasons):

```text
    Mo    Sa  Su
Jan ╆━━━━━┪───┢━━━┪     Expression:
    ┃ (1) ┃   ┃(1)┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
Jul ┨╌╌╌╌╌┃───┣━━━┫
    ┃ (1) ┃   ┃(2)┃     Time rules:
Sep ┨╌╌╌╌╌┃───┣━━━┫     (1) 10:00-18:00
    ┃ (1) ┃   ┃(1)┃     (2) 10:00-22:00
    ┗━━━━━┛───┗━━━┛
```

## Extract covering rectangles out of the table

Second, the algorithm will extract maximal rectangle in the table with all
inner cells equal to the same value.

```text
Step 1: extracted a rectangle
- weekday: Mo-Fr
- month: Jan-Dec
- time: 10:00-18:00

    Mo    Sa  Su
Jan ╆━━━━━┪───┢━━━┓     Expression:
    ┃▚▚▚▚▚┃   ┃(1)┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
Jul ┨▚▚▚▚▚┃───┣━━━┫
    ┃▚▚▚▚▚┃   ┃(2)┃     Time rules:
Sep ┨▚▚▚▚▚┃───┣━━━┫     (1) 10:00-18:00
    ┃▚▚▚▚▚┃   ┃(1)┃     (2) 10:00-22:00
    ┗━━━━━┛───┗━━━┛

Step 2: extracted a rectangle
- weekday: Su
- month: Jan-Jun,Sep-Dec
- time: 10:00-18:00

    Mo        Su
Jan ┼─────────┢━━━┓     Expression:
    │         ┃▚▚▚┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
Jul ┤         ┣━━━┫
    │         ┃(2)┃     Time rules:
Sep ┤         ┣━━━┫     (1) 10:00-18:00
    │         ┃▚▚▚┃     (2) 10:00-22:00
    └─────────┗━━━┛

Step 3: extracted a rectangle
- weekday: Su
- month: Jul-Aug
- time: 10:00-22:00

    Mo        Su
    ├─────────┼───┐     Expression:
    │         │   │     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
Jul ┤         ┏━━━┓
    │         ┃▚▚▚┃     Time rules:
Sep ┤         ┗━━━┛     (1) 10:00-18:00
    │         │   │     (2) 10:00-22:00
    └─────────┴───┘
```

The result is then the concatenation : `Mo-Fr 10:00-18:00; Jan-Jun,Sep-Dec Su
10:00-18:00; Jul-Aug Su 10:00-22:00`.

[gh-issues]: https://github.com/remi-dupre/opening-hours-rs/issues
[spec-fallback]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#fallback_rule_separator
[spec-monthday-range]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#monthday_range
[spec-weekday-range]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#weekday_range
[wiki-idempotence]: https://en.wikipedia.org/wiki/Idempotence
