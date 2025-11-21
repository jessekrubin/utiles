# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "ry>=0.0.68"
# ]
# ///
"""mdbook preprocessor to inject build stuff into docs

BASED ON: https://github.com/PyO3/pyo3/blob/main/guide/pyo3_version.py

Replaces:
  - `{{#RY_DOCS_BUILD_TIMESTAMP}}` with build timestamp

"""

import sys

import ry


def _tokens() -> dict[str, str]:
    return {
        "UTILES_DOCS_BUILD_TIMESTAMP": ry.ZonedDateTime.now().to_string(),
    }


TOKENS = {"{{#" + k + "}}": v for k, v in _tokens().items()}


def replace_tokens(content):
    for token, value in TOKENS.items():
        content = content.replace(token, value)
    return content


def replace_section_content(section):
    if not isinstance(section, dict) or "Chapter" not in section:
        return

    # Replace raw and url-encoded forms
    section["Chapter"]["content"] = replace_tokens(section["Chapter"]["content"])
    for sub_item in section["Chapter"]["sub_items"]:
        replace_section_content(sub_item)


def main():
    for line in sys.stdin:
        if line:
            [_context, book] = ry.JSON.parse(line)
            for section in book["items"]:
                replace_section_content(section)
            sys.stdout.buffer.write(ry.stringify(book))


if __name__ == "__main__":
    main()
