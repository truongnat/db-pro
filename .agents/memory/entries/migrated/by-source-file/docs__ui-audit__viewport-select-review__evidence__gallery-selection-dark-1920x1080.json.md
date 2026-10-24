---
schema_version: 1
id: "MEM-SRC-440aac93abd44743"
title: "Source mirror: docs/ui-audit/viewport-select-review/evidence/gallery-selection-dark-1920x1080.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/ui-audit/viewport-select-review/evidence/gallery-selection-dark-1920x1080.json"
source_sha256: "440aac93abd44743a476f57f3ccc85d7215850d5f7d2f54e34c4726484f8c59e"
mirror_body_sha256: "6c78b714f0be042f74b25890c82495a804f7c5c33aca6e4c2d23ae0b53ff2611"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "issues": [
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|A132|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "A132"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|10F1|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "10F1"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|3513|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "3513"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|C862|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "C862"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|4D56|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "4D56"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|3C4E|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "3C4E"
    },
    {
      "confidence": "high",
      "evidence": "widget enabled=true but semantic node disabled=true",
      "key": "semantic.disabled_mismatch|8922|",
      "other": null,
      "rule": "semantic.disabled_mismatch",
      "severity": "warning",
      "widget": "8922"
    }
  ],
  "meta": {
    "binary": "db-pro-native",
    "fail_policy": "error",
    "fixture": false,
    "frames": 7,
    "layout_fingerprint": 8536424485778705191,
    "os": "macos",
    "scenario": "gallery-selection-dark-1920x1080",
    "theme": "Dark",
    "timestamp": 1792713356,
    "tool": "db-pro-ui-audit",
    "viewport": "1920x1080"
  },
  "rules": [
    {
      "coverage": {
        "evaluated": 300,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "geometry.invalid",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 163,
        "not_applicable": 0,
        "skip_reason": "clipped",
        "skipped": 61
      },
      "id": "interactive.zero_size",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 13041,
        "not_applicable": 0,
        "skip_reason": "clipped",
        "skipped": 8153
      },
      "id": "overlap.suspicious",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 163,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "semantic.missing_name",
      "issues": 6,
      "verdict": "fail"
    },
    {
      "coverage": {
        "evaluated": 163,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "semantic.disabled_mismatch",
      "issues": 1,
      "verdict": "fail"
    },
    {
      "coverage": {
        "evaluated": 309,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "semantic.orphan_node",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 1,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "focus.orphaned",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 1,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "focus.unfocusable",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 3,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "visual.typography_hierarchy",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 4,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "visual.text_contrast",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 34,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "visual.touching_controls",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 0,
        "not_applicable": 1,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "visual.alignment_drift",
      "issues": 0,
      "verdict": "not_applicable"
    }
  ],
  "schema_version": 4,
  "screen": {
    "coverage": {
      "gaps": [],
      "semantic_available": true
    },
    "findings": [
      {
        "confidence": "medium",
        "evidence": "largest region covers 8% of the screen across 37 regions — no dominant work surface",
        "kind": "objective",
        "region": 0,
        "rule": "screen.no_primary_region",
        "severity": "warning",
        "widget": null,
        "widget_id": null
      }
    ],
    "metrics": {
      "balance": {
        "interactive_area_share": {
          "left_bottom": 0.13994194567203522,
          "left_top": 0.3030083477497101,
          "right_bottom": 0.40050938725471497,
          "right_top": 0.15654022991657257
        }
      },
      "density": {
        "interactive": 163,
        "interactive_coverage": 0.3257128894329071,
        "widget_coverage": 1.0,
        "widgets": 300
      },
      "hierarchy": {
        "container_nodes": 100,
        "max_depth": 19,
        "nodes": 310
      },
      "regions": [
        {
          "fill": 0.4876655340194702,
          "floating": false,
          "grid_like": false,
          "interactive": 6,
          "kind": "content",
          "rect": {
            "h": 181.0,
            "w": 726.6875,
            "x": 681.0,
            "y": 265.0
          },
          "screen_share": 0.08174873143434525,
          "widgets": 6
        },
        {
          "fill": 0.7235597372055054,
          "floating": false,
          "grid_like": true,
          "interactive": 18,
          "kind": "side_panel",
          "rect": {
            "h": 599.5,
            "w": 196.0,
            "x": 326.0,
            "y": 89.0
          },
          "screen_share": 0.0730297863483429,
          "widgets": 18
        },
        {
          "fill": 0.4302465319633484,
          "floating": false,
          "grid_like": false,
          "interactive": 6,
          "kind": "content",
          "rect": {
            "h": 106.5,
            "w": 1094.0,
            "x": 681.0,
            "y": 237.5
          },
          "screen_share": 0.0724138543009758,
          "widgets": 6
        },
        {
          "fill": 0.875,
          "floating": false,
          "grid_like": false,
          "interactive": 7,
          "kind": "content",
          "rect": {
            "h": 208.0,
            "w": 240.0,
            "x": 56.0,
            "y": 212.0
          },
          "screen_share": 0.03102625347673893,
          "widgets": 7
        },
        {
          "fill": 0.4916083514690399,
          "floating": false,
          "grid_like": false,
          "interactive": 6,
          "kind": "content",
          "rect": {
            "h": 85.0,
            "w": 473.53125,
            "x": 324.0,
            "y": 42.0
          },
          "screen_share": 0.025016257539391518,
          "widgets": 6
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 1094.03125,
            "x": 681.0,
            "y": 551.5
          },
          "screen_share": 0.024478623643517494,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 1094.03125,
            "x": 681.0,
            "y": 595.5
          },
          "screen_share": 0.024478623643517494,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 1094.03125,
            "x": 681.0,
            "y": 639.5
          },
          "screen_share": 0.024478623643517494,
          "widgets": 1
        },
        {
          "fill": 0.5031116604804993,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 37.5,
            "w": 480.3125,
            "x": 668.0,
            "y": 171.0
          },
          "screen_share": 0.01119463425129652,
          "widgets": 2
        },
        {
          "fill": 0.8931217193603516,
          "floating": false,
          "grid_like": false,
          "interactive": 5,
          "kind": "top_bar",
          "rect": {
            "h": 30.0,
            "w": 504.0,
            "x": 1404.0,
            "y": 4.0
          },
          "screen_share": 0.009397374466061592,
          "widgets": 5
        },
        {
          "fill": 0.4808638393878937,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 37.5,
            "w": 374.78125,
            "x": 668.0,
            "y": 485.0
          },
          "screen_share": 0.008735019713640213,
          "widgets": 2
        },
        {
          "fill": 1.3459957838058472,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "content",
          "rect": {
            "h": 38.0,
            "w": 359.34375,
            "x": 1415.65625,
            "y": 323.53125
          },
          "screen_share": 0.008486887440085411,
          "widgets": 3
        },
        {
          "fill": 0.7669982314109802,
          "floating": false,
          "grid_like": false,
          "interactive": 6,
          "kind": "status_bar",
          "rect": {
            "h": 17.5,
            "w": 601.375,
            "x": 20.0,
            "y": 815.25
          },
          "screen_share": 0.0065409098751842976,
          "widgets": 6
        },
        {
          "fill": 0.6128014922142029,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 37.5,
            "w": 266.375,
            "x": 668.0,
            "y": 708.5
          },
          "screen_share": 0.006208396982401609,
          "widgets": 2
        },
        {
          "fill": 0.6724025011062622,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 235.9375,
            "x": 926.375,
            "y": 775.0
          },
          "screen_share": 0.005279031116515398,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 20.0,
            "w": 359.34375,
            "x": 1415.65625,
            "y": 413.53125
          },
          "screen_share": 0.0044667827896773815,
          "widgets": 1
        },
        {
          "fill": 0.7042548060417175,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 198.46875,
            "x": 1305.375,
            "y": 775.0
          },
          "screen_share": 0.004440679214894772,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 28.0,
            "w": 240.0,
            "x": 56.0,
            "y": 88.0
          },
          "screen_share": 0.004176611080765724,
          "widgets": 1
        },
        {
          "fill": 0.9159663915634155,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 26.0,
            "w": 238.0,
            "x": 56.0,
            "y": 54.0
          },
          "screen_share": 0.0038459626957774162,
          "widgets": 2
        },
        {
          "fill": 0.8294202089309692,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 36.0,
            "w": 138.3125,
            "x": 681.0,
            "y": 775.0
          },
          "screen_share": 0.003094700863584876,
          "widgets": 2
        },
        {
          "fill": 0.5644217729568481,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "status_bar",
          "rect": {
            "h": 26.0,
            "w": 185.75,
            "x": 1722.25,
            "y": 812.0
          },
          "screen_share": 0.0030016284435987473,
          "widgets": 3
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "status_bar",
          "rect": {
            "h": 18.0,
            "w": 260.0,
            "x": 637.375,
            "y": 815.0
          },
          "screen_share": 0.0029087113216519356,
          "widgets": 1
        },
        {
          "fill": 0.8068559169769287,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "top_bar",
          "rect": {
            "h": 17.5,
            "w": 178.9375,
            "x": 142.59375,
            "y": 10.25
          },
          "screen_share": 0.0019462300697341561,
          "widgets": 2
        },
        {
          "fill": 0.8701146245002747,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 28.0,
            "w": 106.6875,
            "x": 1793.3125,
            "y": 94.0
          },
          "screen_share": 0.0018566340440884233,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 14.5,
            "w": 167.40625,
            "x": 1415.65625,
            "y": 386.28125
          },
          "screen_share": 0.0015086706262081861,
          "widgets": 1
        },
        {
          "fill": 0.8666666746139526,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "content",
          "rect": {
            "h": 90.0,
            "w": 26.0,
            "x": 11.0,
            "y": 50.0
          },
          "screen_share": 0.0014543556608259678,
          "widgets": 3
        },
        {
          "fill": 0.9176024794578552,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "content",
          "rect": {
            "h": 15.71875,
            "w": 148.0,
            "x": 64.0,
            "y": 140.15625
          },
          "screen_share": 0.001445887377485633,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 14.5,
            "w": 142.28125,
            "x": 1415.65625,
            "y": 305.03125
          },
          "screen_share": 0.0012822432909160852,
          "widgets": 1
        },
        {
          "fill": 0.8387096524238586,
          "floating": false,
          "grid_like": false,
          "interactive": 2,
          "kind": "top_bar",
          "rect": {
            "h": 26.0,
            "w": 62.0,
            "x": 12.0,
            "y": 6.0
          },
          "screen_share": 0.0010018894681707025,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 13.5,
            "w": 81.46875,
            "x": 60.0,
            "y": 181.25
          },
          "screen_share": 0.0006835646345280111,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 26.0,
            "w": 26.0,
            "x": 11.0,
            "y": 165.0
          },
          "screen_share": 0.0004201471747364849,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 26.0,
            "w": 26.0,
            "x": 11.0,
            "y": 760.0
          },
          "screen_share": 0.0004201471747364849,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 26.0,
            "w": 26.0,
            "x": 236.0,
            "y": 135.0
          },
          "screen_share": 0.0004201471747364849,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 26.0,
            "w": 26.0,
            "x": 270.0,
            "y": 135.0
          },
          "screen_share": 0.0004201471747364849,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 17.5,
            "w": 16.0,
            "x": 556.0,
            "y": 92.25
          },
          "screen_share": 0.00017402545199729502,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 14.0,
            "w": 14.0,
            "x": 1761.0,
            "y": 386.53125
          },
          "screen_share": 0.0001218178222188726,
          "widgets": 1
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "top_bar",
          "rect": {
            "h": 14.5,
            "w": 2.59375,
            "x": 124.0,
            "y": 11.75
          },
          "screen_share": 0.0000233749597100541,
          "widgets": 1
        }
      ],
      "rhythm": {
        "gaps_measured": 56,
        "mode_gap": 6.0,
        "mode_share": 0.3214285671710968,
        "outliers": 8
      }
    },
    "sufficient": true
  },
  "signature": {
    "fixture": false,
    "scenario": "gallery-selection-dark-1920x1080",
    "schema_version": 4,
    "theme": "Dark",
    "viewport": "1920x1080",
    "widget_ids": [
      89834765721377098,
      102903051298566394,
      135393235647425217,
      149864383589571098,
      249079596309457530,
      253807841919882160,
      389274186101224645,
      425721809005972297,
      519062425146072387,
      691298400209209307,
      707185785704006718,
      842590062815196110,
      876561781063412018,
      938760323259034775,
      976475975398739181,
      1005149788699933759,
      1112042238307408491,
      1453901779173913875,
      1554404117555874902,
      1575866342518665557,
      1693163998759462886,
      1696320177534619194,
      1770995255392947902,
      1838841804701453654,
      2048054452826506693,
      2104978516988535048,
      2140188580821309150,
      2177925643082275280,
      2241447118083283071,
      2332450564784046613,
      2385290331737086412,
      2516544803697803369,
      2601960455564797492,
      2664672658996166555,
      2726157836091701603,
      2739755515712588726,
      2850356910134589273,
      2863986438812633049,
      2908075491401821896,
      2946918837933493228,
      3216165712252725689,
      3257461815377231897,
      3263289349858714448,
      3307510860742315961,
      3448386488045837592,
      3580238382977798770,
      3743688473513496817,
      3864622956282445358,
      3920399731243596796,
      4012179969316460122,
      4026534959619162786,
      4041898447145666729,
      4044127923502116639,
      4162282078093467971,
      4163313118271713043,
      4205812211724629461,
      4228775222932014954,
      4447739316592798941,
      4491868723337318047,
      4502096212906009792,
      4568221367500022933,
      4592276652021604962,
      4608950585809288814,
      4677220670256658338,
      4767077446930261037,
      4802217321279561655,
      4835255949838897047,
      4907509085867893486,
      5036820339526962679,
      5081842656791925946,
      5140821107786815902,
      5227686884845628386,
      5244583560739318075,
      5376950469156472477,
      5438354266441836408,
      5492186492276660502,
      5559012291130033053,
      5636961374489715354,
      5698330644590167584,
      5741502929163153355,
      5793746885977356237,
      5847146318001974207,
      5914511678538959081,
      5985175963200642218,
      6071160034427530819,
      6165938459819477496,
      6176075799358672813,
      6180579369714014480,
      6182895444146327949,
      6250340226413586055,
      6265218841284829112,
      6308691343043100552,
      6336047906221566334,
      6486347637874831920,
      6596434109915238734,
      6627053926771750438,
      6709526628867308532,
      6711376841287147029,
      6798660188920138126,
      6910895183926339097,
      6940832320740051684,
      7020504380772750077,
      7040922401288508846,
      7049375824897949656,
      7071236419241532931,
      7133770182896827196,
      7190313569888246528,
      7428796164212850692,
      7469554588109361774,
      7492027308196382408,
      7504998399522335120,
      7580043391292242451,
      7592335268624069406,
      7598676978977554179,
      7647041219756907954,
      7658712652185795586,
      7680484119462703005,
      7682736407040599206,
      7733813648406602002,
      7842109803935464664,
      7850787225469115732,
      7903208410680982269,
      7911809076402445379,
      7993150473536980907,
      8017896671373712410,
      8034067850750876141,
      8322242549291684845,
      8337351045239461913,
      8385670769082289242,
      8407582944683422699,
      8420319761171245574,
      8452302571965489145,
      8531651308044943438,
      8589100763801481531,
      8635049596544620345,
      8639417109827676051,
      8885583843665758203,
      8987097102117408284,
      8999756904481429647,
      9020154431277632210,
      9035376433857410659,
      9088687345774660864,
      9129214544391663591,
      9161244939943165034,
      9229232007335142650,
      9280025966322548013,
      9437423956440839857,
      9517705450657221469,
      9536042742927213594,
      9814064785191626713,
      9996326187117219975,
      10053702121760776169,
      10174422251055133775,
      10295250120950758142,
      10313565819889105040,
      10320920734212739267,
      10386390121699600234,
      10428489328970942535,
      10568410139760408755,
      10630397883856919804,
      10659072448847832694,
      10699172267116701446,
      10735571023708641078,
      10777979668385987845,
      10802138461491195790,
      11203922851306772140,
      11364777321903034914,
      11379150879570618901,
      11379876958851700239,
      11441893856521363904,
      11446146501338805114,
      11489029659484988985,
      11622826464321231071,
      11648019283955539018,
      11834933989832361385,
      11854788468836770098,
      11901921244510588997,
      11955617077649820664,
      12051196415608247445,
      12133656884827152061,
      12179058956545662443,
      12193835670554264907,
      12229331291942629989,
      12293917259930954282,
      12312357417698219929,
      12339000898876861210,
      12405180858539870946,
      12419343155100682812,
      12561234510948692905,
      12611523720487589367,
      12670851110925575215,
      12673343313654287243,
      12993221395597139745,
      13028355941643340043,
      13066891322991538644,
      13195907353229314146,
      13201820095519105784,
      13231518488786118328,
      13233894934006248533,
      13285003157076841291,
      13374004384144218539,
      13420860198817859136,
      13460355476213575492,
      13465824648780381054,
      13480903808659814339,
      13616371190852794595,
      13644167665018976176,
      13654737749681377268,
      13659011098723346518,
      13659180956457199160,
      13711193994602786554,
      13825867788211396204,
      13829768505688570256,
      13830872878472173385,
      13917884443105636896,
      14016034116591247691,
      14074260178150255002,
      14099476584935947829,
      14134958906129518194,
      14219657880291404812,
      14298115738860503126,
      14302880469832395109,
      14332996070375799713,
      14411445691018446454,
      14437600631474128262,
      14525109231877049602,
      14533908937763404351,
      14614723821222950115,
      14652038764605396485,
      14684801279225805901,
      14738464248902738566,
      14741626646762231308,
      14751566895345522837,
      14753005339164457283,
      14856205416441739222,
      14885581197701209981,
      14894502008963189554,
      14955696781011815512,
      14971414359815839627,
      15021739259088297549,
      15087596328762847504,
      15160105658442559625,
      15167109732558240111,
      15358973472424731903,
      15377132529619937261,
      15521108705532646966,
      15521916554531633290,
      15706289773131067213,
      15711900765789652999,
      15780163226702814948,
      15807522644403092830,
      15811320347368732759,
      15817083010816978675,
      15818474963040604834,
      15869317680370822175,
      15875279601797890028,
      15924658052108508380,
      16001076722751817990,
      16020373748451536087,
      16034063879537194360,
      16046819261711557160,
      16069234856383995574,
      16085322632643622052,
      16088083423846625649,
      16088948216526924643,
      16131667669072441310,
      16211254643333773329,
      16244456896666952324,
      16416553767422540142,
      16435602248369015074,
      16442053569990756605,
      16595871122876087374,
      16618076674074792123,
      16663428168025794612,
      16689798197060592735,
      16698754257831230856,
      16710873996954878838,
      16735400269100293099,
      16745515139811148146,
      16892565436223632651,
      16913503664967018983,
      17128685188534667651,
      17251058536885693543,
      17376330698445535233,
      17400944630943893243,
      17450328361385725381,
      17545128470232112916,
      17549915113012494140,
      17846107326714679049,
      17855930079510364366,
      17927971045065277879,
      17939211002146535315,
      18150364454533804840,
      18177122262509635080,
      18200115136194971077,
      18228290013027981390,
      18311152048929305863,
      18342946932515053334,
      18356943641707106210,
      18404808540947117815
    ]
  },
  "summary": {
    "errors": 6,
    "issues": 7,
    "rules": 12,
    "verdicts": {
      "fail": 2,
      "not_applicable": 1,
      "pass": 9,
      "skip": 0
    },
    "warnings": 1,
    "widgets": 300
  },
  "visual_review": {
    "bundle": "/private/tmp/dbpro-ui-layout-audit-6x4vhz4x/evidence/gallery-selection-dark-1920x1080.bundle.json",
    "note": "no provider configured (DB_PRO_AI_REVIEW_CMD unset) — bundle written for later review",
    "screenshot": "gallery-selection-dark-1920x1080.png",
    "status": "bundle_only"
  }
}
```
