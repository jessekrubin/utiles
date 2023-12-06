import warnings

import utiles


def test_check_build_profile() -> None:
    assert (
        utiles.__build_profile__ == "debug" or utiles.__build_profile__ == "release"
    ), f"utiles.__build_profile__ is not 'debug'/'release': {utiles.__build_profile__}"
    if utiles.__build_profile__ == "debug":
        # warn that this is a debug build
        warnings.warn("utiles is built in debug mode", UserWarning, stacklevel=2)
