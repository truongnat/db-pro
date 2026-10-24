---
schema_version: 1
id: "MEM-SRC-17fd8284e9220c5e"
title: "Source mirror: docs/plans/completed/repository-folder-mapping/MAPPING.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/repository-folder-mapping/MAPPING.json"
source_sha256: "17fd8284e9220c5e80316477ccbe7de5a418897348cc3e6570c23411bdbdd118"
mirror_body_sha256: "ef882ed6ce24587fcfd63981958fb9b1d5926c7c8fe05165c320bb85a073051f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "baseline_sha": "be2e431b07a8fcfbf39bf16fa1deefd600273633",
  "source": [
    {
      "from": "crates/ui/src/components/input/tests.rs",
      "to": "crates/ui/src/components/input/tests/mod_tests.rs",
      "sha256_before": "58daa54e6e6ca5189a449f6c07b662aa38dc71f7fdf98aa0279322a926567ca9",
      "bytes_before": 19152
    },
    {
      "from": "crates/ui/src/components/select/tests.rs",
      "to": "crates/ui/src/components/select/tests/mod_tests.rs",
      "sha256_before": "0a9746cad15dab460e5f1f6d548f874317cde20610d65fc13eed073891a6fff4",
      "bytes_before": 2490
    },
    {
      "from": "crates/ui/src/components/dialog/tests.rs",
      "to": "crates/ui/src/components/dialog/tests/mod_tests.rs",
      "sha256_before": "b3b39e59333a892339a7051a8ede0f00b951b9e401521a6422dc37844f728081",
      "bytes_before": 1363
    },
    {
      "from": "crates/ui/src/components/form/tests.rs",
      "to": "crates/ui/src/components/form/tests/mod_tests.rs",
      "sha256_before": "1dd66c85a15fbad149412d7db07772c71d6e1cff6c7f5486454fb81bf22a05a5",
      "bytes_before": 5140
    },
    {
      "from": "crates/ui/src/components/table/tests.rs",
      "to": "crates/ui/src/components/table/tests/mod_tests.rs",
      "sha256_before": "fd1ff2b549438f567de39e5260b3cb0e240c6ef97d1b251d5a9e64d9a0063036",
      "bytes_before": 1946
    },
    {
      "from": "crates/ui/src/components/tabs/tests.rs",
      "to": "crates/ui/src/components/tabs/tests/mod_tests.rs",
      "sha256_before": "ac02877440a408ac22f1f530bfafb68d1f8aadd54b64029a21c336e8744bf4d9",
      "bytes_before": 5600
    },
    {
      "from": "crates/ui/src/diagram/tests.rs",
      "to": "crates/ui/src/diagram/tests/mod_tests.rs",
      "sha256_before": "e09b735d8c11a66862f54ffa3fbc0214195ecb3adc0d05e96db0e5fbe228a7c0",
      "bytes_before": 62299
    },
    {
      "from": "crates/app/src/connection/tests.rs",
      "to": "crates/app/src/connection/tests/mod_tests.rs",
      "sha256_before": "bf139dfd649fcfa30d740c222eb52b573b3bfec4fb3f3e597129e6c28c86adea",
      "bytes_before": 25022
    },
    {
      "from": "crates/core/src/application/schema_service_tests.rs",
      "to": "crates/core/src/application/tests/schema_service_tests.rs",
      "sha256_before": "6572ed9f508430ca5a7586d2d46afc255ca45fb5d94fb2249e3e3dcb7e2b48e7",
      "bytes_before": 38446
    },
    {
      "from": "crates/core/src/application/query_service/tests.rs",
      "to": "crates/core/src/application/tests/query_service_tests.rs",
      "sha256_before": "06d505299f6894a2f256100e22efde88f45fb574c772e88c1abfa3b09b1a443d",
      "bytes_before": 28864
    },
    {
      "from": "crates/core/src/application/connection_service/tests.rs",
      "to": "crates/core/src/application/tests/connection_service_tests.rs",
      "sha256_before": "7be59c39cb40a84650ae382563ef4fc22e2dde8801db658c50a9f70c4a865dea",
      "bytes_before": 26150
    },
    {
      "from": "crates/core/src/application/connection_service/tests/connection_service_security_tests.rs",
      "to": "crates/core/src/application/tests/connection_service_security_tests.rs",
      "sha256_before": "14267d8f45a5ab54b32f60a2108525572e67d553da7cf73b3dcd1fcc69bc95d4",
      "bytes_before": 3483
    },
    {
      "from": "crates/core/src/application/connection_service/tests/connection_service_provider_tests.rs",
      "to": "crates/core/src/application/tests/connection_service_provider_tests.rs",
      "sha256_before": "a487d70e219a406d75595c679e5bddc3ed9b905f934a1301741ff18b6e008479",
      "bytes_before": 4118
    },
    {
      "from": "crates/core/src/application/connection_service/tests/connection_service_lifecycle_tests.rs",
      "to": "crates/core/src/application/tests/connection_service_lifecycle_tests.rs",
      "sha256_before": "12eda3dafa68b4e59931575e40c984e11cc9ffa277d0f18677f3832ced5d134a",
      "bytes_before": 2355
    },
    {
      "from": "crates/core/src/domain/safety/tests.rs",
      "to": "crates/core/src/domain/tests/safety_tests.rs",
      "sha256_before": "e7cde5da3c92f699505b54870f338327f1cebeff04bb14aeb5bc58410c938de8",
      "bytes_before": 20533
    },
    {
      "from": "crates/core/src/domain/agent_workflow/tests.rs",
      "to": "crates/core/src/domain/tests/agent_workflow_tests.rs",
      "sha256_before": "07b71d568eed00590624fa216f86265fce966d8c0e05d500e96aedd9f2de14f6",
      "bytes_before": 7401
    },
    {
      "from": "crates/native-app/src/translate_tests.rs",
      "to": "crates/native-app/src/tests/translate_tests.rs",
      "sha256_before": "33fcd299a9523f47a0137494ea194fc71c67b37e69042b200f23950eebc28706",
      "bytes_before": 11795
    },
    {
      "from": "crates/core/src/domain/capability_limitations.rs",
      "to": "crates/core/src/domain/capabilities/capability_limitations.rs",
      "sha256_before": "58f11609619b54f40eba13ecef5c2d173fa4a55cb2086cd600c6c237c21d0a3a",
      "bytes_before": 7334
    },
    {
      "from": "crates/core/src/domain/capability_presets.rs",
      "to": "crates/core/src/domain/capabilities/capability_presets.rs",
      "sha256_before": "e12dc3dfaa577934142670773688e83b670dc88b42079344bb94af6a9c277798",
      "bytes_before": 11510
    },
    {
      "from": "crates/core/src/domain/safety_cte.rs",
      "to": "crates/core/src/domain/safety/safety_cte.rs",
      "sha256_before": "73e7514eb6d1a781768e08ce97e7edd7c9aa8ac23c442c9009ad243f4006d4dc",
      "bytes_before": 7473
    },
    {
      "from": "crates/core/src/domain/safety_lexer.rs",
      "to": "crates/core/src/domain/safety/safety_lexer.rs",
      "sha256_before": "369d8fd4ed4f64bd8a04ea225588780e624fb6bf973aa3673fdfa04f029c3379",
      "bytes_before": 7549
    },
    {
      "from": "crates/core/src/domain/safety_policy.rs",
      "to": "crates/core/src/domain/safety/safety_policy.rs",
      "sha256_before": "de2837689210e335f8ed57a433384ab79841e9156eff74ffe8db19ec4824d361",
      "bytes_before": 2583
    },
    {
      "from": "crates/core/src/application/multi_query_execution.rs",
      "to": "crates/core/src/application/query_service/multi_query_execution.rs",
      "sha256_before": "97260b23fce161aac370eed0a9adb232401c21ec706d9c7aaeaaf00b8dc33bf7",
      "bytes_before": 6583
    },
    {
      "from": "crates/core/src/application/query_classification.rs",
      "to": "crates/core/src/application/query_service/query_classification.rs",
      "sha256_before": "648f5c46da5c2aace502e67d62e550cc4c89d76541a4089ba796e261251047d4",
      "bytes_before": 3789
    },
    {
      "from": "crates/core/src/application/schema_ddl.rs",
      "to": "crates/core/src/application/schema_service/schema_ddl.rs",
      "sha256_before": "cb6386b9cc3ad702f65c9ebd2cf57cb6b4a6395495476779d2ed4fc983f6b0fd",
      "bytes_before": 10647
    },
    {
      "from": "crates/core/src/application/schema_table_info.rs",
      "to": "crates/core/src/application/schema_service/schema_table_info.rs",
      "sha256_before": "28e7ddddf79dfa4bac98a3429c608ce221852363b23d02c9b21096860dd742aa",
      "bytes_before": 8775
    },
    {
      "from": "crates/native-app/src/translate.rs",
      "to": "crates/native-app/src/translate/mod.rs",
      "sha256_before": "134ebb6ee7bc29b4d054f7b999256f923ce620e2990ced72a614569b560f1887",
      "bytes_before": 347
    },
    {
      "from": "crates/native-app/src/translate_cmd.rs",
      "to": "crates/native-app/src/translate/translate_cmd.rs",
      "sha256_before": "d523c6a58df0bc779c3eb9aac6d41813624381338032c6bb1dfc8cbb09c5f230",
      "bytes_before": 41156
    },
    {
      "from": "crates/native-app/src/tests/translate_cmd_tests.rs",
      "to": "crates/native-app/src/translate/tests/translate_cmd_tests.rs",
      "sha256_before": "b16028566acfb333101215e3838313e7cfa43e2e2727219066888228814d74cb",
      "bytes_before": 3335
    },
    {
      "from": "crates/native-app/src/translate_evt.rs",
      "to": "crates/native-app/src/translate/translate_evt.rs",
      "sha256_before": "59b5ca9499f9e8116b5d3425224126f03ae67b7e2e48908205df72b043bffa3e",
      "bytes_before": 21081
    },
    {
      "from": "crates/native-app/src/tests/translate_evt_tests.rs",
      "to": "crates/native-app/src/translate/tests/translate_evt_tests.rs",
      "sha256_before": "76eecdbbe6eb84b91e3fabfa646c38835c2936866c986d1ecf753f698a3fe2ff",
      "bytes_before": 6054
    },
    {
      "from": "crates/native-app/src/translate_map.rs",
      "to": "crates/native-app/src/translate/translate_map.rs",
      "sha256_before": "278f6bc633b3ee2e1dc3256c97e61f8eef57c46e7c483b599667c7f9f03ef01e",
      "bytes_before": 14663
    },
    {
      "from": "crates/native-app/src/tests/translate_map_tests.rs",
      "to": "crates/native-app/src/translate/tests/translate_map_tests.rs",
      "sha256_before": "718d05f5774c2327b3afdc43f57c3d22f8c1c05fa03e0b087f23d41b3e6c6b4b",
      "bytes_before": 1585
    },
    {
      "from": "crates/runtime/src/agent.rs",
      "to": "crates/runtime/src/agent/mod.rs",
      "sha256_before": "9e808ecbb53ef9884e91ebd6333638446282970d7e719f70f60d37335d0dcbf2",
      "bytes_before": 29625
    },
    {
      "from": "crates/runtime/src/agent_executor.rs",
      "to": "crates/runtime/src/agent/executor.rs",
      "sha256_before": "00ab34f54879c50d6665c8b49bfef93493a7a9b6d60943fea067652d2d6ce25b",
      "bytes_before": 23376
    },
    {
      "from": "crates/runtime/src/agent_orchestrator.rs",
      "to": "crates/runtime/src/agent/orchestrator.rs",
      "sha256_before": "6c87f6b2948c74e051a9e304f42bfd44242d9562ad3cf062fb7262ff4c5ac45f",
      "bytes_before": 28691
    },
    {
      "from": "crates/runtime/src/tests/agent_tests.rs",
      "to": "crates/runtime/src/agent/tests/agent_tests.rs",
      "sha256_before": "7eee834c18c666133b4b2c70303fbe49923bd5698f13d86fe3483664bd3e68e8",
      "bytes_before": 4733
    },
    {
      "from": "crates/runtime/src/tests/agent_executor_tests.rs",
      "to": "crates/runtime/src/agent/tests/agent_executor_tests.rs",
      "sha256_before": "72cc64fe601b553591eff6e388849c8368144033eeb9dd952639d3d399d3ed9d",
      "bytes_before": 4393
    },
    {
      "from": "crates/runtime/src/tests/agent_orchestrator_tests.rs",
      "to": "crates/runtime/src/agent/tests/agent_orchestrator_tests.rs",
      "sha256_before": "c463829707ac0354117ad8f24ec35f4f8e9d712f911ca27c9cb35251124cb8b8",
      "bytes_before": 29060
    },
    {
      "from": "crates/app/src/tools/problems_view.rs",
      "to": "crates/app/src/shell/sidebar/problems_view.rs",
      "sha256_before": "540f3e86d67a0808e69561b2f093f692510fa5b824740882086b394aa09dc625",
      "bytes_before": 11734
    }
  ],
  "documentation": [
    {
      "from": "plans/00-index.md",
      "to": "docs/plans/project/00-index.md",
      "sha256_before": "f32a560eeca95a643c1303eea3c9965864074d375f16dda1ab06660314a84a38",
      "bytes_before": 4556
    },
    {
      "from": "plans/01-overview.md",
      "to": "docs/plans/project/01-overview.md",
      "sha256_before": "13316bc78206c788b65c79b32b694e5e603c13f73972086fbd49d9cb577d5693",
      "bytes_before": 7499
    },
    {
      "from": "plans/02-backend-tasks.md",
      "to": "docs/plans/project/02-backend-tasks.md",
      "sha256_before": "b526b03b51dc78ec09f9f0d2b6034e3c0134a03121912534838d3f67bad649a9",
      "bytes_before": 14146
    },
    {
      "from": "plans/03-frontend-tasks.md",
      "to": "docs/plans/project/03-frontend-tasks.md",
      "sha256_before": "0747663ae1ae8a4a6687b3bfea617c2fbf975ee368f9d5591f0df5379cd72515",
      "bytes_before": 23050
    },
    {
      "from": "plans/04-testing-tasks.md",
      "to": "docs/plans/project/04-testing-tasks.md",
      "sha256_before": "18ae640d3b3eacb94f295820520902ef0c9c6a8b1fbd9c70496f3671ad172272",
      "bytes_before": 10632
    },
    {
      "from": "plans/05-cicd-tasks.md",
      "to": "docs/plans/project/05-cicd-tasks.md",
      "sha256_before": "d0ca66b292511ef9f8a99adc4fbb369fc327c0a0ade29ecf0ff8cfc1b8bec669",
      "bytes_before": 7754
    },
    {
      "from": "plans/06-database-tasks.md",
      "to": "docs/plans/project/06-database-tasks.md",
      "sha256_before": "a88dffbc73a413e28c831c51d43ccf6abb4441373bf48ad333d99a030d5f7109",
      "bytes_before": 13562
    },
    {
      "from": "plans/07-current-status.md",
      "to": "docs/plans/project/07-current-status.md",
      "sha256_before": "0f276e84d995e4c75bf8b8ea3e937f3defbfffedeb74f71ed25fecd22912c5a9",
      "bytes_before": 16021
    },
    {
      "from": "tasks/plan.md",
      "to": "docs/plans/project/core-component-standardization/plan.md",
      "sha256_before": "9cf1a2bc8a45879afdc9b6d2d89ad0c4f903cc31f071a0e5cd315fb983ac8cda",
      "bytes_before": 5076
    },
    {
      "from": "tasks/todo.md",
      "to": "docs/plans/project/core-component-standardization/todo.md",
      "sha256_before": "e73ba1928c9123ac0df02d6a4fec72deeb40f948889186ed33df7aca5c26c0db",
      "bytes_before": 17358
    },
    {
      "from": "QA_AUDIT_REPORT.md",
      "to": "docs/plans/active/bsn-real-data-native-audit/QA_AUDIT_REPORT.md",
      "sha256_before": "12a60925266b1370d6f139f4e265f8334d4313535c46d6d2b49c288fba3bf800",
      "bytes_before": 40220
    },
    {
      "from": "CLEAN_CODE_AUDIT_REPORT.md",
      "to": "docs/quality/CLEAN_CODE_AUDIT_REPORT.md",
      "sha256_before": "4a213df999508ce242b7406e85937f7f824c4971ab05bf962220b36985599fd1",
      "bytes_before": 30191
    },
    {
      "from": "open-api-style.md",
      "to": "docs/design/reference/open-api-style.md",
      "sha256_before": "6fe4112f270e8694488c0bec14dfe8c1113a3612fdac846f6011c1668a882041",
      "bytes_before": 19577
    },
    {
      "from": "open-ai-refer.md",
      "to": "docs/design/reference/open-ai-refer.md",
      "sha256_before": "825f417760cd9ad58af8aac06bdf711d243b5fcf229980fd64e0303ab504addf",
      "bytes_before": 15111
    },
    {
      "from": "docs/07-fe-architecture.md",
      "to": "docs/architecture/07-fe-architecture.md",
      "sha256_before": "f508f95dbbccf6e480ed16046f64e5a441232e21580d84675d7939930a9daaa7",
      "bytes_before": 5264
    },
    {
      "from": "docs/08-technology-decisions.md",
      "to": "docs/architecture/08-technology-decisions.md",
      "sha256_before": "f266bd264e51862effe41ed03e78b56b1dd299dc46b8f697a5bc92d69ff81b3f",
      "bytes_before": 19370
    },
    {
      "from": "docs/09-architecture-decisions.md",
      "to": "docs/architecture/09-architecture-decisions.md",
      "sha256_before": "6361bbc7585a273aa42729ac30a82962feebf8013b9ff158a6b3e3f50fbe95a4",
      "bytes_before": 11165
    },
    {
      "from": "docs/rendering-analysis.md",
      "to": "docs/architecture/rendering-analysis.md",
      "sha256_before": "21b750f00e224dff5ebc49fef520fe66750933415303594188154666888bfc33",
      "bytes_before": 8362
    },
    {
      "from": "docs/rendering-history.md",
      "to": "docs/architecture/rendering-history.md",
      "sha256_before": "a77692a7ba8811a02c3d5d95b33f2863a71dc933a13d7ee604dc59b76edaf7a6",
      "bytes_before": 3790
    },
    {
      "from": "docs/dev-tools-audit-engine.md",
      "to": "docs/ui-audit/dev-tools-audit-engine.md",
      "sha256_before": "301fb0f64c363b3e190f368eea3a9bed5b637320aa9b8d8fc19af85a1a786846",
      "bytes_before": 36537
    },
    {
      "from": "docs/dev-tools-audit-engine-vi.md",
      "to": "docs/ui-audit/dev-tools-audit-engine-vi.md",
      "sha256_before": "0b3d82643345a210c0ef1e146ae7b76833c4b2f0a7a3e01c0dc321c65ad2f36e",
      "bytes_before": 35001
    },
    {
      "from": "docs/introspection-audit-2026-08-13.md",
      "to": "docs/quality/introspection-audit-2026-08-13.md",
      "sha256_before": "36e2a4b4d8418bedb8710d0b1f5f3fe3230605fc84ea77298ef877c152672a6c",
      "bytes_before": 6718
    },
    {
      "from": "docs/screen_1_sql_editor.html",
      "to": "docs/design/mockups/screen_1_sql_editor.html",
      "sha256_before": "550fc485028220fad025dc904b08a51be0b4a6fca3ddf7b7f67658cec7ee73cc",
      "bytes_before": 66195
    },
    {
      "from": "docs/screen_component_library.html",
      "to": "docs/design/mockups/screen_component_library.html",
      "sha256_before": "fe262fc635956be5bc6452b9ea6773010a6995297a64ac4f9db150c191b93de3",
      "bytes_before": 69062
    },
    {
      "from": "docs/screen_explorer.html",
      "to": "docs/design/mockups/screen_explorer.html",
      "sha256_before": "6c36b597d43c93ad8cba9a0c9a8e6a43bdab26be7aab1e80ed97ebe1dc4e0fd7",
      "bytes_before": 37547
    },
    {
      "from": "docs/plans/dead-code-analysis-2026-08-13.md",
      "to": "docs/plans/archived/dead-code-analysis-2026-08-13.md",
      "sha256_before": "f12412dbde3efd55c2b6a292938e38e3129568556e64b51ba49196041abf2a77",
      "bytes_before": 4357
    },
    {
      "from": "docs/plans/performance-baseline-audit-2026-08-13.md",
      "to": "docs/plans/archived/performance-baseline-audit-2026-08-13.md",
      "sha256_before": "0616b0ca9538e5a3037ac6f017320ba8201941e3a333e29648086e7b71066953",
      "bytes_before": 4804
    },
    {
      "from": "docs/plans/performance-optimization-report-2026-08-13.md",
      "to": "docs/plans/archived/performance-optimization-report-2026-08-13.md",
      "sha256_before": "b5de725d3c3e16340ea5c057fdd284ce7b065b5a4f46f81406905f13e8749276",
      "bytes_before": 5336
    },
    {
      "from": "docs/stitch_screens/01_db_pro___sql_editor_and_results_grid_native_theme_fixed.html",
      "to": "docs/design/stitch_screens/01_db_pro___sql_editor_and_results_grid_native_theme_fixed.html",
      "sha256_before": "ab5d7cf41ba007ea2469a5238d1ef787cb86fa4405d52bd739771d468353d75d",
      "bytes_before": 40392
    },
    {
      "from": "docs/stitch_screens/02_db_pro___sql_editor_and_results_grid_native_theme.html",
      "to": "docs/design/stitch_screens/02_db_pro___sql_editor_and_results_grid_native_theme.html",
      "sha256_before": "e58936928db6b4368d70bb605820b7a1528835814fe266c1c9aefa4c39227673",
      "bytes_before": 56298
    },
    {
      "from": "docs/stitch_screens/03_db_pro___er_diagram_audit_schema.html",
      "to": "docs/design/stitch_screens/03_db_pro___er_diagram_audit_schema.html",
      "sha256_before": "49ae3e2c201b91f14a9ddce5aaebf99fbd0e0e1b6ce9771d9f02cf53827c19d1",
      "bytes_before": 36354
    },
    {
      "from": "docs/stitch_screens/04_db_pro___data_activity_f1_stable_table_reference_identity.html",
      "to": "docs/design/stitch_screens/04_db_pro___data_activity_f1_stable_table_reference_identity.html",
      "sha256_before": "18f5bc534439d5a51a37c732e26cadc24c4214edec226944f616170b7f9fa87c",
      "bytes_before": 36369
    },
    {
      "from": "docs/stitch_screens/05_db_pro_component_library___inputs_selection_and_navigation.html",
      "to": "docs/design/stitch_screens/05_db_pro_component_library___inputs_selection_and_navigation.html",
      "sha256_before": "363a19cd87d5a24da42e2425aff9c2fd7e5bb517243ad2e5e3fef9c9d75debb1",
      "bytes_before": 59549
    },
    {
      "from": "docs/stitch_screens/06_designmd.html",
      "to": "docs/design/stitch_screens/06_designmd.html",
      "sha256_before": "23ff808cd79a242f52a847664df1284a38924dc86c60f75949655738d9648539",
      "bytes_before": 909
    },
    {
      "from": "docs/stitch_screens/07_db_pro_component_library___sql_editor_and_results_grid.html",
      "to": "docs/design/stitch_screens/07_db_pro_component_library___sql_editor_and_results_grid.html",
      "sha256_before": "550fc485028220fad025dc904b08a51be0b4a6fca3ddf7b7f67658cec7ee73cc",
      "bytes_before": 66195
    },
    {
      "from": "docs/stitch_screens/08_db_pro___sql_editor_and_results_grid_native_theme.html",
      "to": "docs/design/stitch_screens/08_db_pro___sql_editor_and_results_grid_native_theme.html",
      "sha256_before": "80864b3c9e1089f87efbfb2387bdcc141cf5d958d6b64753f3d333b00ad1f7d5",
      "bytes_before": 2844
    },
    {
      "from": "docs/stitch_screens/09_db_pro___schema_compare_f1_block_incomplete_migration_ddl.html",
      "to": "docs/design/stitch_screens/09_db_pro___schema_compare_f1_block_incomplete_migration_ddl.html",
      "sha256_before": "b09f7bd71c936057b0e5a99aa8eb1bc744d477819ea5f63f5be38b11f917630c",
      "bytes_before": 31928
    },
    {
      "from": "docs/stitch_screens/10_db_pro___er_diagram_f1_schema_scoped_graph_refresh.html",
      "to": "docs/design/stitch_screens/10_db_pro___er_diagram_f1_schema_scoped_graph_refresh.html",
      "sha256_before": "5e0b9ca38ea7193c657137550d0b86a2598d995cf1c837651f0b84e012b1f2da",
      "bytes_before": 49680
    },
    {
      "from": "docs/stitch_screens/11_db_pro___explorer_f1_object_filter_workbench.html",
      "to": "docs/design/stitch_screens/11_db_pro___explorer_f1_object_filter_workbench.html",
      "sha256_before": "6c36b597d43c93ad8cba9a0c9a8e6a43bdab26be7aab1e80ed97ebe1dc4e0fd7",
      "bytes_before": 37547
    },
    {
      "from": "docs/stitch_screens/12_db_pro___execution_history_f1_unified_execution_history.html",
      "to": "docs/design/stitch_screens/12_db_pro___execution_history_f1_unified_execution_history.html",
      "sha256_before": "ea0a8b5b46cc173198045ac677551da106b62f299c73f48cb25a4a09fed0d908",
      "bytes_before": 29406
    },
    {
      "from": "docs/stitch_screens/13_db_pro___execution_history_proposed_f1.html",
      "to": "docs/design/stitch_screens/13_db_pro___execution_history_proposed_f1.html",
      "sha256_before": "435a93122018930feaf9c9875c3f0f48297559319b2a03fd59dbf7f2bb488652",
      "bytes_before": 24645
    },
    {
      "from": "docs/stitch_screens/14_db_pro___queries_activity_f1_safe_open_and_connection_context.html",
      "to": "docs/design/stitch_screens/14_db_pro___queries_activity_f1_safe_open_and_connection_context.html",
      "sha256_before": "9642f768191bd2337329c49c72a27947970824de54d2997eb0a34af708d83449",
      "bytes_before": 34866
    },
    {
      "from": "docs/stitch_screens/15_db_pro___files_workspace_f1_trust_and_file_mutation_review.html",
      "to": "docs/design/stitch_screens/15_db_pro___files_workspace_f1_trust_and_file_mutation_review.html",
      "sha256_before": "1a534a1396e834d6e7318f930940e8c655f1bc396e615766c1ce60e12d430757",
      "bytes_before": 31323
    },
    {
      "from": "docs/stitch_screens/16_db_pro___rebind_a_missing_table_reference.html",
      "to": "docs/design/stitch_screens/16_db_pro___rebind_a_missing_table_reference.html",
      "sha256_before": "9fec6319d1700964f8cd997171126a11d8e38c8b899a74563462d8a4e90f3cab",
      "bytes_before": 36045
    },
    {
      "from": "docs/stitch_screens/17_db_pro___queries_activity_f1_source_context_unavailable.html",
      "to": "docs/design/stitch_screens/17_db_pro___queries_activity_f1_source_context_unavailable.html",
      "sha256_before": "baf2ffdeb1963a64b3eb39d1db9803afed616dd2c31c0b24bbb864f25c775dbc",
      "bytes_before": 29772
    },
    {
      "from": "docs/stitch_screens/18_db_pro_component_library___feedback_dialogs_and_commands.html",
      "to": "docs/design/stitch_screens/18_db_pro_component_library___feedback_dialogs_and_commands.html",
      "sha256_before": "5c1a2ced12624d1333b388591b9fad701650d293e4ef82aa9ff5edd3e75ef49e",
      "bytes_before": 61189
    },
    {
      "from": "docs/stitch_screens/19_db_pro_component_library.html",
      "to": "docs/design/stitch_screens/19_db_pro_component_library.html",
      "sha256_before": "fe262fc635956be5bc6452b9ea6773010a6995297a64ac4f9db150c191b93de3",
      "bytes_before": 69062
    },
    {
      "from": "docs/stitch_screens/inventory.json",
      "to": "docs/design/stitch_screens/inventory.json",
      "sha256_before": "bf30243a2ffeec55b17bdb68a151ea47c94bdfa5c19ca35ae059b917fc546c32",
      "bytes_before": 5263
    },
    {
      "from": "screenshots/query-workspace-zed-shell/A-untitled-1280x800.png",
      "to": "docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-1280x800.png",
      "sha256_before": "0deaa3cbfbb5606e2264a08648902f09f1b08293b04796183a12d0182d4521dc",
      "bytes_before": 128530
    },
    {
      "from": "screenshots/query-workspace-zed-shell/A-untitled-1440x900.png",
      "to": "docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-1440x900.png",
      "sha256_before": "cd4d23a8cd86c450daca069fe487b496e9830929ac91bca74bc595217439d00b",
      "bytes_before": 133273
    },
    {
      "from": "screenshots/query-workspace-zed-shell/A-untitled-dark-1280x800.png",
      "to": "docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-dark-1280x800.png",
      "sha256_before": "ae8ec6e7d4a054e17242100f0816cdd15685cc8ddaafb3894cc0f6c16740f79b",
      "bytes_before": 149898
    },
    {
      "from": "screenshots/query-workspace-zed-shell/theme-dark-1280x800.png",
      "to": "docs/plans/completed/query-workspace-zed-shell/screenshots/theme-dark-1280x800.png",
      "sha256_before": "e66bb093cc40210a7ad29ba32335be0b52c9d7cf70372b707c817c25e41bcd18",
      "bytes_before": 138684
    },
    {
      "from": "screenshots/query-workspace-zed-shell/theme-light-1280x800.png",
      "to": "docs/plans/completed/query-workspace-zed-shell/screenshots/theme-light-1280x800.png",
      "sha256_before": "58cba9a82ed6b0a766cca5a0d77b0b93ccf810bf8be441a8c92dfe81f462f99f",
      "bytes_before": 137190
    }
  ]
}

```
