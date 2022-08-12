This modules basically builds a data-structure for a set of days based on
bit-maps. This is built to store a collection of regional holidays for the
[opening-hours] crate.


Data layout
-----------

Here is how serialized data is represented:

```text
 start   size       year 1          year 2      ...
┌──────┬──────┬───────────────┬───────────────┬────
│  8B  │  8B  │ 8B * 12 = 96B │ 8B * 12 = 96B │ ...
└──────┴──────┴───────────────┴───────────────┴────
```

Each year is just an array of 12 [`u32`] where the least significant bits each
represent a day.

While a bitset might not be the most efficient way to store a collection of
dates for sparse data, this approached proved to be very compact when combined
with a Zlib encoder. This methods allowed to store all holidays from 2000 to
2100 as described by [workalendar] in only 60kb of data.


[opening-hours]: https://docs.rs/opening-hours/latest/opening_hours
[workalendar]: https://github.com/workalendar/workalendar
