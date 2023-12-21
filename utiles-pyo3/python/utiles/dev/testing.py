"""Utiles rust cli tests"""
from __future__ import annotations

try:
    from orjson import loads as json_loads
except ImportError:
    from json import loads as json_loads

import sys
from dataclasses import dataclass
from subprocess import CompletedProcess, run
from time import time_ns
from typing import Any


@dataclass
class CliResult:
    """CLI result"""

    __slots__ = (
        "args",
        "stdout",
        "stderr",
        "returncode",
        "dt",
        "input",
        "completed_process",
    )

    args: list[str]
    stdout: str
    stderr: str
    returncode: int

    # time in seconds
    dt: float
    # input
    input: str | None
    completed_process: CompletedProcess[str] | None

    def __str__(self) -> str:
        fields = (
            "args",
            "stdout",
            "stderr",
            "returncode",
            "dt",
            "input",
        )
        parts = (f"{f}={getattr(self, f)}" for f in fields)
        return f"CliResult({', '.join(parts)})"

    @property
    def exit_code(self) -> int:
        """Exit code"""
        return self.returncode

    @property
    def output(self) -> str:
        """Success"""
        return self.stdout

    @property
    def parse_json(self) -> Any:
        """Parse json"""
        return json_loads(self.stdout)


def run_cli(
    args: list[str] | None,
    input: str | None = None,
) -> CliResult:
    _python = sys.executable
    _args = args or []
    ti = time_ns()
    completed_process = run(
        [_python, "-m", "utiles.cli", *_args],
        input=input,
        capture_output=True,
        text=True,
        shell=False,  # noqa: S603
    )
    tf = time_ns()
    return CliResult(
        args=_args,
        stdout=completed_process.stdout,
        stderr=completed_process.stderr,
        returncode=completed_process.returncode,
        input=input,
        dt=(tf - ti) / 1e9,
        completed_process=completed_process,
    )
