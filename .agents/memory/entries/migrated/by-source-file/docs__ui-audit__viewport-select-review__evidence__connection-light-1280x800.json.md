---
schema_version: 1
id: "MEM-SRC-f713175175893aef"
title: "Source mirror: docs/ui-audit/viewport-select-review/evidence/connection-light-1280x800.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/ui-audit/viewport-select-review/evidence/connection-light-1280x800.json"
source_sha256: "f713175175893aef121842575a7d4f137c79907bd035e07ec0587317c3b83847"
mirror_body_sha256: "b411341f64348643d3ec34f2db3857b914afcd3e3cb2fd0eb30278cd45639f71"
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
      "evidence": "interactable 0.0×0.0px < 1px floor (rect 0.0×0.0)",
      "key": "interactive.zero_size|6570|",
      "other": null,
      "rule": "interactive.zero_size",
      "severity": "error",
      "widget": "6570"
    },
    {
      "confidence": "high",
      "evidence": "Unknown node has no label / labelled_by / value",
      "key": "semantic.missing_name|343A|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "343A"
    },
    {
      "confidence": "high",
      "evidence": "ScrollBar node has no label / labelled_by / value",
      "key": "semantic.missing_name|C689|",
      "other": null,
      "rule": "semantic.missing_name",
      "severity": "error",
      "widget": "C689"
    },
    {
      "confidence": "medium",
      "evidence": "left edge 4.0px off the column's shared edge (255.0), within the 0.5–4px drift band",
      "key": "visual.alignment_drift|7B73|",
      "other": null,
      "rule": "visual.alignment_drift",
      "severity": "warning",
      "widget": "7B73"
    }
  ],
  "meta": {
    "binary": "db-pro-native",
    "fail_policy": "error",
    "fixture": false,
    "frames": 13,
    "layout_fingerprint": 11874993206033085712,
    "os": "macos",
    "scenario": "connection-light-1280x800",
    "theme": "Light",
    "timestamp": 1792713187,
    "tool": "db-pro-ui-audit",
    "viewport": "1280x800"
  },
  "rules": [
    {
      "coverage": {
        "evaluated": 266,
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
        "evaluated": 116,
        "not_applicable": 0,
        "skip_reason": "clipped",
        "skipped": 5
      },
      "id": "interactive.zero_size",
      "issues": 1,
      "verdict": "fail"
    },
    {
      "coverage": {
        "evaluated": 3117,
        "not_applicable": 0,
        "skip_reason": "intentional overlay",
        "skipped": 3733
      },
      "id": "overlap.suspicious",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 114,
        "not_applicable": 0,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "semantic.missing_name",
      "issues": 2,
      "verdict": "fail"
    },
    {
      "coverage": {
        "evaluated": 114,
        "not_applicable": 0,
        "skip_reason": "no semantic data",
        "skipped": 2
      },
      "id": "semantic.disabled_mismatch",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 290,
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
        "evaluated": 15,
        "not_applicable": 0,
        "skip_reason": "intentional overlay",
        "skipped": 48
      },
      "id": "visual.touching_controls",
      "issues": 0,
      "verdict": "pass"
    },
    {
      "coverage": {
        "evaluated": 1,
        "not_applicable": 4,
        "skip_reason": null,
        "skipped": 0
      },
      "id": "visual.alignment_drift",
      "issues": 1,
      "verdict": "fail"
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
        "evidence": "largest region covers 8% of the screen across 20 regions — no dominant work surface",
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
          "left_bottom": 0.020329849794507027,
          "left_top": 0.7194955945014954,
          "right_bottom": 0.03325062617659569,
          "right_top": 0.2269241064786911
        }
      },
      "density": {
        "interactive": 116,
        "interactive_coverage": 1.0,
        "widget_coverage": 1.0,
        "widgets": 266
      },
      "hierarchy": {
        "container_nodes": 116,
        "max_depth": 13,
        "nodes": 291
      },
      "regions": [
        {
          "fill": 0.8965898752212524,
          "floating": false,
          "grid_like": false,
          "interactive": 4,
          "kind": "content",
          "rect": {
            "h": 140.0,
            "w": 620.0,
            "x": 484.0,
            "y": 279.0
          },
          "screen_share": 0.08476562798023224,
          "widgets": 4
        },
        {
          "fill": 0.2808230519294739,
          "floating": false,
          "grid_like": false,
          "interactive": 4,
          "kind": "content",
          "rect": {
            "h": 129.0,
            "w": 462.3125,
            "x": 562.84375,
            "y": 122.0
          },
          "screen_share": 0.05824054032564163,
          "widgets": 4
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
          "fill": 0.9392029047012329,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "content",
          "rect": {
            "h": 32.0,
            "w": 277.5625,
            "x": 324.0,
            "y": 42.0
          },
          "screen_share": 0.008673828095197678,
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
          "fill": 0.5665891170501709,
          "floating": false,
          "grid_like": false,
          "interactive": 3,
          "kind": "status_bar",
          "rect": {
            "h": 26.0,
            "w": 140.5,
            "x": 1127.5,
            "y": 774.0
          },
          "screen_share": 0.003567382926121354,
          "widgets": 3
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
          "kind": "top_bar",
          "rect": {
            "h": 26.0,
            "w": 26.0,
            "x": 12.0,
            "y": 6.0
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
        "gaps_measured": 32,
        "mode_gap": 8.0,
        "mode_share": 0.375,
        "outliers": 6
      }
    },
    "sufficient": true
  },
  "signature": {
    "fixture": false,
    "scenario": "connection-light-1280x800",
    "schema_version": 4,
    "theme": "Light",
    "viewport": "1280x800",
    "widget_ids": [
      89834765721377098,
      253807841919882160,
      309350574567381855,
      346785709746013155,
      428856469972640117,
      464487128459656122,
      472088147201969472,
      489499807461792897,
      573858768825716957,
      611715326022903807,
      624121439954723339,
      691298400209209307,
      977033613806873231,
      1025876346713247955,
      1061871922751169904,
      1089163464641549277,
      1112042238307408491,
      1277755777932387110,
      1468661282194704009,
      1616323964701352564,
      2097040992495059479,
      2104978516988535048,
      2140188580821309150,
      2262233713039145100,
      2367681255810544821,
      2467589229837115906,
      2726157836091701603,
      2739975937346321355,
      2781758364411337743,
      2835429484707307483,
      2863986438812633049,
      2938215778806207240,
      2962794173705505454,
      3181725201517411991,
      3313166469410892960,
      3469020324342726676,
      3477967321823527100,
      3580238382977798770,
      3590048344032994799,
      3702416220348202379,
      4007308192436597818,
      4026534959619162786,
      4044127923502116639,
      4163313118271713043,
      4297840933619538936,
      4401185898045075742,
      4408825297116287195,
      4411422805753155303,
      4460769338035905628,
      4502096212906009792,
      4592276652021604962,
      4662311210750688682,
      4730629233181064936,
      4841181770361248820,
      4891318783802587871,
      4907509085867893486,
      4934004687997407339,
      4971330442345135893,
      4974929084247876466,
      5076489024161670126,
      5086605002685317942,
      5140821107786815902,
      5227686884845628386,
      5244583560739318075,
      5362408371574926754,
      5367537216902764636,
      5418740656381823489,
      5492186492276660502,
      5523238371851101007,
      5559012291130033053,
      5603749094868946068,
      5677271326149717085,
      5698330644590167584,
      5700060360900130450,
      5741502929163153355,
      5778826243449286212,
      5793746885977356237,
      5985175963200642218,
      6071160034427530819,
      6176075799358672813,
      6180579369714014480,
      6238578565456741321,
      6321871948973115824,
      6336047906221566334,
      6392966446406207894,
      6498798450898956324,
      6523960918751418816,
      6579145809329824395,
      6584552404248858602,
      6596434109915238734,
      6601190352249685327,
      6662390284969062529,
      6818727254273514533,
      6851755897650392216,
      6880326024560247925,
      7020504380772750077,
      7054389651150911181,
      7071236419241532931,
      7133770182896827196,
      7428796164212850692,
      7537235479799030622,
      7647041219756907954,
      7658712652185795586,
      7816815232500298272,
      8017896671373712410,
      8039346489222007342,
      8118836148307509111,
      8162048846243378099,
      8227848531964988393,
      8245343287631586398,
      8340782206816626248,
      8393713131895752409,
      8407582944683422699,
      8531651308044943438,
      8639417109827676051,
      8826700751608999367,
      8885583843665758203,
      8973608597001256930,
      8996211231340666290,
      8999756904481429647,
      9035376433857410659,
      9036682962721851415,
      9216560242782586121,
      9280025966322548013,
      9437423956440839857,
      9450394380271739695,
      9520657496294700546,
      9538358320974877855,
      9759963084638063702,
      9845661239700672320,
      9900520821079834937,
      9932704351536999246,
      9950962755665707215,
      10060855835986551188,
      10222680652965840426,
      10295250120950758142,
      10320920734212739267,
      10466613862495761656,
      10699172267116701446,
      10802138461491195790,
      10954000215572201575,
      10978751946435619837,
      11147735155858863163,
      11200716382599858028,
      11203922851306772140,
      11254580623257586835,
      11272243507572118087,
      11299665051015696994,
      11489029659484988985,
      11565624010481957355,
      11622826464321231071,
      11645263547062514097,
      11831834952841549050,
      11844997941839560752,
      11874395436330505755,
      11948695708599191104,
      11955617077649820664,
      11990341901810418465,
      12051196415608247445,
      12110497093948826408,
      12113964011687977801,
      12129231575778085905,
      12193835670554264907,
      12339000898876861210,
      12380626353659443138,
      12405180858539870946,
      12652262764462558183,
      12796054399535203273,
      13057599858206851234,
      13060732372221626682,
      13066891322991538644,
      13074076379948369801,
      13212329543288157303,
      13225103987063341950,
      13273913470653628218,
      13355055854547682905,
      13374004384144218539,
      13465824648780381054,
      13480903808659814339,
      13644167665018976176,
      13654737749681377268,
      13659180956457199160,
      13664149719259906799,
      13697427028252190599,
      13711193994602786554,
      13722234226951982351,
      13754031454478838710,
      13829768505688570256,
      13844057155745010844,
      13917884443105636896,
      14099476584935947829,
      14117702312415727519,
      14219657880291404812,
      14332996070375799713,
      14437600631474128262,
      14533908937763404351,
      14593572902755052809,
      14594178215994552763,
      14615474153664751808,
      14617527089990642421,
      14634922852589541484,
      14652038764605396485,
      14684801279225805901,
      14738464248902738566,
      14753005339164457283,
      14759326841369359867,
      14765308714061610505,
      14885581197701209981,
      14894502008963189554,
      14905004277833322167,
      14971414359815839627,
      15021739259088297549,
      15051851036527867233,
      15358973472424731903,
      15479427090503868672,
      15521108705532646966,
      15568641567237040797,
      15706289773131067213,
      15741021810002404719,
      15746274183626535836,
      15759453692821633156,
      15780163226702814948,
      15783625848869544940,
      15846771611640051821,
      15875279601797890028,
      15924658052108508380,
      15957936226515916513,
      16001076722751817990,
      16046819261711557160,
      16088948216526924643,
      16131667669072441310,
      16239272592019281607,
      16239479420801805697,
      16244456896666952324,
      16317632440326847347,
      16469831396241285484,
      16584757352055829898,
      16597415347675021249,
      16663428168025794612,
      16689798197060592735,
      16694552506338612920,
      16698754257831230856,
      16724087393262356099,
      16735400269100293099,
      16861035176848310956,
      16976480440508247895,
      17001024699338699994,
      17026834222916680249,
      17119987548376132796,
      17376330698445535233,
      17450328361385725381,
      17474153457193692759,
      17549915113012494140,
      17585909952996402777,
      17661961731304112756,
      17670276921834342982,
      17762944152442987505,
      17781091914037863612,
      17853058897601188370,
      17855690669336000664,
      18005190921649460956,
      18228290013027981390,
      18257023905486474744,
      18311152048929305863,
      18356943641707106210,
      18404808540947117815
    ]
  },
  "summary": {
    "errors": 3,
    "issues": 4,
    "rules": 12,
    "verdicts": {
      "fail": 3,
      "not_applicable": 0,
      "pass": 9,
      "skip": 0
    },
    "warnings": 1,
    "widgets": 266
  },
  "visual_review": {
    "bundle": "/private/tmp/dbpro-ui-layout-audit-6x4vhz4x/evidence/connection-light-1280x800.bundle.json",
    "note": "no provider configured (DB_PRO_AI_REVIEW_CMD unset) — bundle written for later review",
    "screenshot": "connection-light-1280x800.png",
    "status": "bundle_only"
  }
}
```
