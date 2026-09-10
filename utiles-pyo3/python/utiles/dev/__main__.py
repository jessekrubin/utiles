"""dev entry point"""

from __future__ import annotations

import json

import utiles
from utiles.__main__ import _lib_info


def _banner() -> str:
    json_info = json.dumps(_lib_info(), indent=2)
    return f"~~~~~~~~~~~~~\nutiles.dev ~ repl\n~~~~~~~~~~~~~\n{json_info}"


def _main() -> None:
    try:
        import rich
        from rich import inspect
        from rich import print as pprint
    except ImportError:
        from pprint import pprint

        rich = inspect = None  # ty:ignore[invalid-assignment]

    # locals
    local = globals()
    local.update(
        {
            "inspect": inspect,
            "pprint": pprint,
            "rich": rich,
            "ut": utiles,
        }
    )
    # everything from utiles
    local.update({k: getattr(utiles, k) for k in dir(utiles)})
    # try to do das IPython first and 4-most...!
    try:
        import sys

        import IPython

        IPython.InteractiveShell.banner1 = _banner()  # type: ignore[attr-defined,assignment]  # ty:ignore[invalid-assignment]
        rich = None  # ty:ignore[invalid-assignment]
        ipython_argv = [
            "--no-tip",
            "--TerminalInteractiveShell.editing_mode=vi",
            "--TerminalInteractiveShell.emacs_bindings_in_vi_insert_mode=False",
            *sys.argv[1:],
        ]
        if rich is not None:
            ipython_argv.extend(["--ext", "rich"])
        IPython.start_ipython(argv=ipython_argv, user_ns=local)
        return
    except ImportError:
        ...

    import code

    code.interact(_banner(), local=local)


if __name__ == "__main__":
    _main()
