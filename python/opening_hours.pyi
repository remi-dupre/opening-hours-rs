from datetime import datetime
from typing import Self

def validate(oh: str) -> bool:
    """
    Validate that input string is a correct opening hours description.

    Examples
    --------
    >>> opening_hours.validate("24/7")
    True
    >>> opening_hours.validate("24/24")
    False
    """

class RangeIterator:
    def __iter__(self) -> Self:
        ...

    def __next__(self) -> tuple[datetime, datetime]:
        ...

class OpeningHours:
    """
    Class for parsing input opening hours description.

    Raises
    ------
    SyntaxError
        Given string is not in valid opening hours format.

    Examples
    --------
    >>> oh = OpeningHours("24/7")
    >>> oh.is_open()
    True
    """

    def __init__(self, oh: str) -> None:
        ...

    def __str__(self) -> str:
        ...

    def __repr__(self) -> str:
        ...

    def state(self, dt: datetime | None = None) -> str:
        """
        Get current state of the time domain, the state can be either "open",
        "closed" or "unknown".

        Parameters
        ----------
        time : Optional[datetime]
            Base time for the evaluation, current time will be used if it is
            not specified.

        Examples
        --------
        >>> OpeningHours("24/7 off").state()
        'closed'
        """

    def is_open(self, dt: datetime | None = None) -> bool:
        """
        Check if current state is open.

        Parameters
        ----------
        time : Optional[datetime]
            Base time for the evaluation, current time will be used if it is
            not specified.

        Examples
        --------
        >>> OpeningHours("24/7").is_open()
        True
        """

    def is_closed(self, dt: datetime | None = None) -> bool:
        """
        Check if current state is closed.

        Parameters
        ----------
        time : Optional[datetime]
            Base time for the evaluation, current time will be used if it is
            not specified.

        Examples
        --------
        >>> OpeningHours("24/7 off").is_closed()
        True
        """

    def is_unknown(self, dt: datetime | None = None) -> bool:
        """
        Check if current state is unknown.

        Parameters
        ----------
        time : Optional[datetime]
            Base time for the evaluation, current time will be used if it is
            not specified.

        Examples
        --------
        >>> OpeningHours("24/7 unknown").is_unknown()
        True
        """

    def next_change(self, dt: datetime | None = None) -> datetime:
        """
        Get the date for next change of state.
        If the date exceed the limit date, returns None.

        Parameters
        ----------
        time : Optional[datetime]
            Base time for the evaluation, current time will be used if it is
            not specified.

        Examples
        --------
        >>> OpeningHours("24/7").next_change() # None
        >>> OpeningHours("2099Mo-Su 12:30-17:00").next_change()
        datetime.datetime(2099, 1, 1, 12, 30)
        """

    def intervals(self, start: datetime | None = None, end: datetime | None = None) -> RangeIterator:
        """
        Give an iterator that yields successive time intervals of consistent
        state.

        Parameters
        ----------
        start: Optional[datetime]
            Initial time for the iterator, current time will be used if it is
            not specified.
        end : Optional[datetime]
            Maximal time for the iterator, the iterator will continue until
            year 9999 if it no max is specified.

        Examples
        --------
        >>> intervals = OpeningHours("2099Mo-Su 12:30-17:00").intervals()
        >>> next(intervals)
        (..., datetime.datetime(2099, 1, 1, 12, 30), 'closed', [])
        >>> next(intervals)
        (datetime.datetime(2099, 1, 1, 12, 30), datetime.datetime(2099, 1, 1, 17, 0), 'open', [])
        """
