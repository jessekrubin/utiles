import json

import utiles as ut
from utiles.dev.testing import run_cli as _run_cli

TEXTILES_INPUT = """
[4188, 3104, 13]
[4192, 2977, 13]
[4192, 3098, 13]
[4192, 2983, 13]
[4192, 2935, 13]
[4192, 2982, 13]
[4192, 2980, 13]
[4192, 3101, 13]
[4192, 2987, 13]
[4192, 2987, 13]
[4192, 2986, 13]
[4192, 2981, 13]
[4192, 2997, 13]
[4192, 2969, 13]
[4192, 2947, 13]
[4192, 2927, 13]
[4192, 2961, 13]
[4192, 2988, 13]
[4192, 2976, 13]
[4192, 2891, 13]
[4192, 2994, 13]
[4192, 2959, 13]
[4192, 2892, 13]
[4192, 2975, 13]
[4192, 2931, 13]
[4192, 2943, 13]
[4192, 2971, 13]
[4192, 2931, 13]
[4192, 2919, 13]
[4192, 2929, 13]
[4192, 2930, 13]
[4192, 2897, 13]
[4192, 2878, 13]
[4192, 2879, 13]
[4192, 2980, 13]
[4192, 2868, 13]
[4192, 2887, 13]
[4192, 2881, 13]
[4192, 2913, 13]
[4192, 2884, 13]
[4192, 2899, 13]
[4192, 2809, 13]
[4192, 2859, 13]
[4192, 2807, 13]
[4192, 2921, 13]
[4192, 2775, 13]
[4192, 2811, 13]
[4192, 2827, 13]
[4192, 2867, 13]
[4192, 2865, 13]
[4192, 2856, 13]
[4192, 2873, 13]
[4192, 2863, 13]
[4192, 2839, 13]
[4192, 2774, 13]
[4192, 2974, 13]
[4192, 2808, 13]
[4192, 2832, 13]
[4192, 2793, 13]
[4192, 3098, 13]
[4192, 2787, 13]
[4192, 2859, 13]
[4192, 2853, 13]
[4192, 2825, 13]
[4192, 2825, 13]
[4192, 2808, 13]
[4192, 2787, 13]
[4192, 2898, 13]
[4192, 2812, 13]
[4192, 2859, 13]
[4192, 2765, 13]
[4192, 2806, 13]
[4192, 2769, 13]
[4192, 2964, 13]
[4192, 2821, 13]
[4192, 2778, 13]
[4192, 2785, 13]
[4192, 2805, 13]
[4192, 2737, 13]
[4192, 2800, 13]
[4192, 2762, 13]
[4192, 2756, 13]
[4192, 2986, 13]
[4192, 2794, 13]
[4192, 2760, 13]
[4192, 2777, 13]
[4192, 2782, 13]
[4192, 2746, 13]
[4192, 2748, 13]
[4192, 2745, 13]
[4192, 2871, 13]
[4192, 2798, 13]
[4192, 2758, 13]
[4192, 2756, 13]
[4192, 2750, 13]
[4192, 2977, 13]
[4192, 2765, 13]
[4192, 2981, 13]
[4192, 3099, 13]
[4192, 2983, 13]
"""

TEXTILES_EXPECTED = """
[4188, 3104, 13]
[4192, 2737, 13]
[4192, 2745, 13]
[4192, 2746, 13]
[4192, 2748, 13]
[4192, 2750, 13]
[4192, 2756, 13]
[4192, 2758, 13]
[4192, 2760, 13]
[4192, 2762, 13]
[4192, 2765, 13]
[4192, 2769, 13]
[4192, 2774, 13]
[4192, 2775, 13]
[4192, 2777, 13]
[4192, 2778, 13]
[4192, 2782, 13]
[4192, 2785, 13]
[4192, 2787, 13]
[4192, 2793, 13]
[4192, 2794, 13]
[4192, 2798, 13]
[4192, 2800, 13]
[4192, 2805, 13]
[4192, 2806, 13]
[4192, 2807, 13]
[4192, 2808, 13]
[4192, 2809, 13]
[4192, 2811, 13]
[4192, 2812, 13]
[4192, 2821, 13]
[4192, 2825, 13]
[4192, 2827, 13]
[4192, 2832, 13]
[4192, 2839, 13]
[4192, 2853, 13]
[4192, 2856, 13]
[4192, 2859, 13]
[4192, 2863, 13]
[4192, 2865, 13]
[4192, 2867, 13]
[4192, 2868, 13]
[4192, 2871, 13]
[4192, 2873, 13]
[4192, 2878, 13]
[4192, 2879, 13]
[4192, 2881, 13]
[4192, 2884, 13]
[4192, 2887, 13]
[4192, 2891, 13]
[4192, 2892, 13]
[4192, 2897, 13]
[4192, 2898, 13]
[4192, 2899, 13]
[4192, 2913, 13]
[4192, 2919, 13]
[4192, 2921, 13]
[4192, 2927, 13]
[4192, 2929, 13]
[4192, 2930, 13]
[4192, 2931, 13]
[4192, 2935, 13]
[4192, 2943, 13]
[4192, 2947, 13]
[4192, 2959, 13]
[4192, 2961, 13]
[4192, 2964, 13]
[4192, 2969, 13]
[4192, 2971, 13]
[4192, 2974, 13]
[4192, 2975, 13]
[4192, 2976, 13]
[4192, 2977, 13]
[4192, 2980, 13]
[4192, 2981, 13]
[4192, 2982, 13]
[4192, 2983, 13]
[4192, 2986, 13]
[4192, 2987, 13]
[4192, 2988, 13]
[4192, 2994, 13]
[4192, 2997, 13]
[4192, 3098, 13]
[4192, 3099, 13]
[4192, 3101, 13]
"""


def test_cli_edges() -> None:
    edges_result = _run_cli(
        [
            "edges",
        ],
        input=TEXTILES_INPUT,
    )
    parsed_edges = edges_result.parse_tiles

    assert set(parsed_edges) == {
        ut.xyz(*json.loads(e)) for e in TEXTILES_EXPECTED.split("\n") if e.strip()
    }
