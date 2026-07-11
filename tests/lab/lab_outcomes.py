"""Outcome types shared by bounded laboratory probes."""


class Skip(Exception):
    pass


class Blocked(Exception):
    pass
