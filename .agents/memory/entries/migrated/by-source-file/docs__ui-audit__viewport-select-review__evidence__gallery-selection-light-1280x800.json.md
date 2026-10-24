---
schema_version: 1
id: "MEM-SRC-1c92fac0891caafe"
title: "Source mirror: docs/ui-audit/viewport-select-review/evidence/gallery-selection-light-1280x800.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/ui-audit/viewport-select-review/evidence/gallery-selection-light-1280x800.json"
source_sha256: "1c92fac0891caafe403ca0720d7fd0d0a3428339c16e69e20febdc7216fc187c"
mirror_body_sha256: "60a02739e44158c34ce46dd3af7346c77d170acc459fadc60d3b2040f0d34826"
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
      "key": "semantic.missing_name|3C4E|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "3C4E"
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
      "key": "semantic.missing_name|4D56|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "4D56"
    },
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
      "key": "semantic.missing_name|C862|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "C862"
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
    "layout_fingerprint": 10603633533257419754,
    "os": "macos",
    "scenario": "gallery-selection-light-1280x800",
    "theme": "Light",
    "timestamp": 1792713326,
    "tool": "db-pro-ui-audit",
    "viewport": "1280x800"
  },
  "rules": [
    {
      "coverage": {
        "evaluated": 301,
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
        "skipped": 68
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
        "skipped": 8832
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
        "evaluated": 303,
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
        "evidence": "largest region covers 15% of the screen across 33 regions — no dominant work surface",
        "kind": "objective",
        "region": 0,
        "rule": "screen.no_primary_region",
        "severity": "warning",
        "widget": null,
        "widget_id": null
      },
      {
        "confidence": "medium",
        "evidence": "13 of 15 interactive widgets packed within 2px on both axes in a non-data region",
        "kind": "heuristic",
        "region": 0,
        "rule": "screen.control_congestion",
        "severity": "warning",
        "widget": null,
        "widget_id": null
      },
      {
        "confidence": "medium",
        "evidence": "11 of 11 interactive widgets packed within 2px on both axes in a non-data region",
        "kind": "heuristic",
        "region": 2,
        "rule": "screen.control_congestion",
        "severity": "warning",
        "widget": null,
        "widget_id": null
      }
    ],
    "metrics": {
      "balance": {
        "interactive_area_share": {
          "left_bottom": 0.1689988374710083,
          "left_top": 0.20598207414150238,
          "right_bottom": 0.4195365011692047,
          "right_top": 0.2054826319217682
        }
      },
      "density": {
        "interactive": 163,
        "interactive_coverage": 0.40177637338638306,
        "widget_coverage": 1.0,
        "widgets": 301
      },
      "hierarchy": {
        "container_nodes": 101,
        "max_depth": 19,
        "nodes": 304
      },
      "regions": [
        {
          "fill": 0.31812652945518494,
          "floating": false,
          "grid_like": false,
          "interactive": 7,
          "kind": "content",
          "rect": {
            "h": 224.5,
            "w": 670.0,
            "x": 573.0,
            "y": 265.0
          },
          "screen_share": 0.14688964188098907,
          "widgets": 7
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
          "screen_share": 0.1147480458021164,
          "widgets": 18
        },
        {
          "fill": 0.3969552218914032,
          "floating": false,
          "grid_like": false,
          "interactive": 6,
          "kind": "content",
          "rect": {
            "h": 150.0,
            "w": 670.0,
            "x": 573.0,
            "y": 237.5
          },
          "screen_share": 0.09814453125,
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
          "screen_share": 0.04874999821186066,
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
          "screen_share": 0.03930679336190224,
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
            "w": 670.0,
            "x": 573.0,
            "y": 595.0
          },
          "screen_share": 0.023554688319563866,
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
            "w": 670.0,
            "x": 573.0,
            "y": 639.0
          },
          "screen_share": 0.023554688319563866,
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
            "w": 670.0,
            "x": 573.0,
            "y": 683.0
          },
          "screen_share": 0.023554688319563866,
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
            "x": 560.0,
            "y": 171.0
          },
          "screen_share": 0.017589569091796875,
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
            "x": 764.0,
            "y": 4.0
          },
          "screen_share": 0.014765624888241291,
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
            "x": 560.0,
            "y": 528.5
          },
          "screen_share": 0.013724898919463158,
          "widgets": 2
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
            "y": 777.25
          },
          "screen_share": 0.010277404449880123,
          "widgets": 6
        },
        {
          "fill": 1.3428747653961182,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "content",
          "rect": {
            "h": 38.0,
            "w": 218.0,
            "x": 1025.0,
            "y": 323.53125
          },
          "screen_share": 0.008089844137430191,
          "widgets": 3
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
          "screen_share": 0.006562499795109034,
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
          "screen_share": 0.006042968947440386,
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
            "x": 1082.25,
            "y": 774.0
          },
          "screen_share": 0.0047163087874650955,
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
            "y": 777.0
          },
          "screen_share": 0.004570312332361937,
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
          "screen_share": 0.0030580139718949795,
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
            "x": 1153.3125,
            "y": 94.0
          },
          "screen_share": 0.002917236415669322,
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
            "x": 1025.0,
            "y": 386.28125
          },
          "screen_share": 0.0023704986087977886,
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
          "screen_share": 0.0022851561661809683,
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
          "screen_share": 0.0022718505933880806,
          "widgets": 2
        },
        {
          "fill": 1.0,
          "floating": false,
          "grid_like": false,
          "interactive": 1,
          "kind": "content",
          "rect": {
            "h": 17.0,
            "w": 132.875,
            "x": 560.0,
            "y": 752.0
          },
          "screen_share": 0.0022059325128793716,
          "widgets": 1
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
            "x": 1025.0,
            "y": 305.03125
          },
          "screen_share": 0.0020147247705608606,
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
          "screen_share": 0.001574218738824129,
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
          "screen_share": 0.00107405090238899,
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
          "screen_share": 0.0006601562490686774,
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
            "y": 722.0
          },
          "screen_share": 0.0006601562490686774,
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
          "screen_share": 0.0006601562490686774,
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
          "screen_share": 0.0006601562490686774,
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
          "screen_share": 0.0002734375011641532,
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
            "x": 1229.0,
            "y": 386.53125
          },
          "screen_share": 0.00019140624499414116,
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
          "screen_share": 0.000036727906262967736,
          "widgets": 1
        }
      ],
      "rhythm": {
        "gaps_measured": 56,
        "mode_gap": 6.0,
        "mode_share": 0.3035714328289032,
        "outliers": 8
      }
    },
    "sufficient": true
  },
  "signature": {
    "fixture": false,
    "scenario": "gallery-selection-light-1280x800",
    "schema_version": 4,
    "theme": "Light",
    "viewport": "1280x800",
    "widget_ids": [
      89834765721377098,
      107173590493254350,
      135393235647425217,
      149864383589571098,
      208251090131450212,
      253807841919882160,
      389274186101224645,
      519062425146072387,
      691298400209209307,
      842590062815196110,
      876561781063412018,
      882620686873496894,
      938760323259034775,
      976475975398739181,
      1112042238307408491,
      1453901779173913875,
      1554404117555874902,
      1575866342518665557,
      1693163998759462886,
      1696320177534619194,
      1838841804701453654,
      1852529390727317490,
      2104978516988535048,
      2140188580821309150,
      2177925643082275280,
      2235506884823059801,
      2241447118083283071,
      2385290331737086412,
      2516544803697803369,
      2601960455564797492,
      2664672658996166555,
      2716074402381756115,
      2726157836091701603,
      2739755515712588726,
      2795306297171307141,
      2850356910134589273,
      2863986438812633049,
      2908075491401821896,
      2962914641494573694,
      2998878276945899246,
      3215335363814458281,
      3216165712252725689,
      3263289349858714448,
      3307510860742315961,
      3370777286910554959,
      3429275500883491819,
      3448386488045837592,
      3474974747656872188,
      3580238382977798770,
      3743688473513496817,
      3864622956282445358,
      4005742186630053211,
      4012179969316460122,
      4026534959619162786,
      4041898447145666729,
      4044127923502116639,
      4162282078093467971,
      4163313118271713043,
      4205812211724629461,
      4228775222932014954,
      4340771607706912672,
      4502096212906009792,
      4568221367500022933,
      4592276652021604962,
      4608950585809288814,
      4621415353335634548,
      4690880070798126090,
      4767077446930261037,
      4835255949838897047,
      4907509085867893486,
      4972108865733473412,
      5007249865694429496,
      5036820339526962679,
      5081842656791925946,
      5140821107786815902,
      5227686884845628386,
      5244583560739318075,
      5425945137765478435,
      5492186492276660502,
      5559012291130033053,
      5559481711139888030,
      5628652964353034177,
      5698330644590167584,
      5741502929163153355,
      5793746885977356237,
      5804225423572817151,
      5847146318001974207,
      5914511678538959081,
      5985175963200642218,
      6071160034427530819,
      6165938459819477496,
      6176075799358672813,
      6180579369714014480,
      6218036028578875451,
      6250340226413586055,
      6336047906221566334,
      6398561552647897236,
      6486347637874831920,
      6596434109915238734,
      6627053926771750438,
      6709526628867308532,
      6711376841287147029,
      6819734476755628845,
      6910895183926339097,
      6940832320740051684,
      7020504380772750077,
      7040922401288508846,
      7049375824897949656,
      7071236419241532931,
      7133770182896827196,
      7146202165196862223,
      7190313569888246528,
      7321214265051443600,
      7343920537646593962,
      7428796164212850692,
      7575231230253054490,
      7580043391292242451,
      7592335268624069406,
      7647041219756907954,
      7650027165902831326,
      7658712652185795586,
      7680484119462703005,
      7682736407040599206,
      7733813648406602002,
      7842109803935464664,
      7850787225469115732,
      7903208410680982269,
      7911809076402445379,
      8017896671373712410,
      8238671152347573569,
      8337351045239461913,
      8385670769082289242,
      8407582944683422699,
      8452302571965489145,
      8527634695657618934,
      8531651308044943438,
      8543215483083931541,
      8589100763801481531,
      8639417109827676051,
      8712402034179287269,
      8780500010809389031,
      8819661967389874087,
      8885583843665758203,
      8999756904481429647,
      9035376433857410659,
      9094490013328343087,
      9129214544391663591,
      9229232007335142650,
      9280025966322548013,
      9437423956440839857,
      9463985576711339361,
      9517705450657221469,
      9536042742927213594,
      9814064785191626713,
      9892931456501410714,
      10040855829980105005,
      10154905994586034083,
      10239170879180771962,
      10295250120950758142,
      10320920734212739267,
      10375590822781133334,
      10428489328970942535,
      10568410139760408755,
      10630397883856919804,
      10699172267116701446,
      10802138461491195790,
      11203922851306772140,
      11364777321903034914,
      11379150879570618901,
      11379876958851700239,
      11446146501338805114,
      11489029659484988985,
      11622826464321231071,
      11648019283955539018,
      11798653859341919500,
      11854788468836770098,
      11896836967831691569,
      11955617077649820664,
      12039222697306799364,
      12051196415608247445,
      12119359370446178345,
      12133656884827152061,
      12193835670554264907,
      12229331291942629989,
      12312357417698219929,
      12339000898876861210,
      12405180858539870946,
      12419343155100682812,
      12501184564541487326,
      12611523720487589367,
      12670851110925575215,
      12673343313654287243,
      12696976837892489917,
      12926050522238028404,
      12993221395597139745,
      13066891322991538644,
      13078021977911408865,
      13195907353229314146,
      13201820095519105784,
      13215403729095013360,
      13231518488786118328,
      13285003157076841291,
      13374004384144218539,
      13465824648780381054,
      13480903808659814339,
      13616371190852794595,
      13644167665018976176,
      13654737749681377268,
      13659011098723346518,
      13659180956457199160,
      13702554411663955121,
      13711193994602786554,
      13825867788211396204,
      13829768505688570256,
      13862006861350164393,
      13917884443105636896,
      14016034116591247691,
      14099476584935947829,
      14134958906129518194,
      14196094320245393078,
      14219657880291404812,
      14298115738860503126,
      14332996070375799713,
      14398627405499279551,
      14437600631474128262,
      14533908937763404351,
      14594516879080263246,
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
      15022902376415216747,
      15087596328762847504,
      15167109732558240111,
      15358973472424731903,
      15377132529619937261,
      15521108705532646966,
      15521916554531633290,
      15706289773131067213,
      15780163226702814948,
      15811320347368732759,
      15817083010816978675,
      15818474963040604834,
      15869317680370822175,
      15869534791372611355,
      15875279601797890028,
      15924658052108508380,
      15968928269461199931,
      16001076722751817990,
      16020373748451536087,
      16046819261711557160,
      16069234856383995574,
      16085322632643622052,
      16088083423846625649,
      16088948216526924643,
      16131667669072441310,
      16240713263444896389,
      16244456896666952324,
      16321007916946601486,
      16348150009422687701,
      16409709701454701521,
      16435602248369015074,
      16595871122876087374,
      16607107372465008969,
      16663428168025794612,
      16689798197060592735,
      16698754257831230856,
      16718218092198957528,
      16735400269100293099,
      16745515139811148146,
      16804626368384199374,
      16988917888792894536,
      17312525272093566196,
      17376330698445535233,
      17400944630943893243,
      17414533038287420738,
      17415795103017065071,
      17450328361385725381,
      17541241625281422114,
      17545128470232112916,
      17549915113012494140,
      17846107326714679049,
      17855930079510364366,
      17927971045065277879,
      17939211002146535315,
      18177122262509635080,
      18200115136194971077,
      18228290013027981390,
      18261358464485785068,
      18311152048929305863,
      18318320800432090129,
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
    "widgets": 301
  },
  "visual_review": {
    "bundle": "/private/tmp/dbpro-ui-layout-audit-6x4vhz4x/evidence/gallery-selection-light-1280x800.bundle.json",
    "note": "no provider configured (DB_PRO_AI_REVIEW_CMD unset) — bundle written for later review",
    "screenshot": "gallery-selection-light-1280x800.png",
    "status": "bundle_only"
  }
}
```
