"""Utiles rust cli tests"""

from __future__ import annotations

import utiles

try:
    from orjson import loads as json_loads
except ImportError:
    from json import loads as json_loads
import sys
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import connect
from subprocess import CompletedProcess, run
from time import time_ns
from typing import Any

echo = print


@dataclass
class CliResult:
    """CLI result"""

    __slots__ = (
        "args",
        "completed_process",
        "dt",
        "input",
        "returncode",
        "stderr",
        "stdout",
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

    def parse_jsonl(self) -> list[Any]:
        """Parse json"""
        return [json_loads(line) for line in self.stdout.splitlines()]

    def parse_tiles(self) -> list[utiles.Tile]:
        """Parse tile lines"""
        return [
            utiles.xyz(arr[0], arr[1], arr[2])
            for arr in (
                json_loads(line) for line in self.stdout.splitlines(keepends=False)
            )
        ]

    def fmt(self) -> str:
        return "\n".join(
            (
                f"args: {self.args}",
                f"stdout: {self.stdout}",
                f"stderr: {self.stderr}",
                f"returncode: {self.returncode}",
                f"dt: {self.dt}",
                f"input: {self.input}",
                f"completed_process: {self.completed_process}",
            )
        )

    def echo(self) -> None:
        """echo the result for testing/debugging"""
        echo(self.fmt())

    def print(self) -> None:
        """Print alias"""
        self.echo()


def run_cli(
    args: list[str] | None,
    input: str | None = None,
) -> CliResult:
    _python = sys.executable
    _args = args or []
    ti = time_ns()
    completed_process = run(  # noqa: S603
        [_python, "-m", "utiles.cli", *_args],
        input=input,
        capture_output=True,
        text=True,
        shell=False,
        check=False,
    )
    tf = time_ns()
    res = CliResult(
        args=_args,
        stdout=completed_process.stdout,
        stderr=completed_process.stderr,
        returncode=completed_process.returncode,
        input=input,
        dt=(tf - ti) / 1e9,
        completed_process=completed_process,
    )
    if res.returncode != 0:
        res.echo()
    return res


def query_metadata_rows(
    dbpath: str | Path,
) -> list[dict[str, Any]]:
    """Query metadata rows"""

    with connect(dbpath) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM metadata;")
        rows = cursor.fetchall()
    return [dict(zip((d[0] for d in cursor.description), row)) for row in rows]
