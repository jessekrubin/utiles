from __future__ import annotations

import json
import os
import subprocess as sp
import sys


def echo(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def get_cmd_help(command_name: str) -> str:
    res = sp.run(["utiles", command_name, "--help"], capture_output=True)
    return res.stdout.decode()


def main():
    cur_env = os.environ.copy()
    # set `UTILES_MAX_TERM_WIDTH` to 80 for docs formating to not look dumb
    cur_env["UTILES_MAX_TERM_WIDTH"] = "80"
    res = sp.run(["utiles", "commands"], capture_output=True, env=cur_env)
    echo(res.stdout.decode())
    commands = json.loads(res.stdout.decode())

    cmd_parts = []
    cmd_toc = [
        "| Command | Description |",
        "| ------- | ----------- |",
    ]

    for command in commands:
        echo(command)

        cmd_help = get_cmd_help(command["name"])
        if command["name"] != command["path"]:
            continue  # not handled currently
        if "UNIMPLEMENTED" in cmd_help:
            continue

        cmd_parts.append("___")
        cmd_parts.append(f"## {command['name']}")
        cmd_parts.append(f"```bash\n{cmd_help}\n```")
        # cmd_toc.append(f"- [{command['name']}](#{command['name']}) {command['about']}")
        cmd_toc.append(
            f"| [{command['name']}](#{command['name']}) | {command['about']} |"
        )

    markdown_parts = [
        "# utiles CLI commands",
        "## Table of contents",
        "\n".join(cmd_toc),
        *(el for el in cmd_parts),
    ]

    with open("docs/src/commands.md", "w") as f:
        f.write("\n\n".join(markdown_parts))


if __name__ == "__main__":
    main()
