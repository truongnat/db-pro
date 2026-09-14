# Keyring startup stall — classification (brief §37)

- Session: `v01-runtime`, 2026-09-14
- Reproduced on: the packaged CI artifact of run `34860902181` (archive
  `db-pro-v0.1.0-macos-arm64.tar.gz`, sha256 `8141d6b7…`, bundled binary `cf40855b…`)
- Raw evidence: `16-launch-a-keyring-stall.txt` (the stall + thread dump),
  `17-launch-bc-state-dir.txt` (the no-item control and the bypass launch)
- Related register entries: `R-KEYRING-STALL` (OPEN, ACCEPTED-with-disclosure),
  `LIM-018`, `R003` (unsigned/ad-hoc-signed artifacts)

## 1. What was run

| Launch | HOME | `GROQ_API_KEY` | Observed |
|---|---|---|---|
| **A** | real | **unset** | **STALLED** — alive at 12 s, blocked inside the Keychain call, never reached the data directory |
| **B** | fake (empty, no keychain) | unset | Passed the keyring immediately, reached `-[NSApplication run]`, created its state directory |
| **C** | real | placeholder (**explicit bypass**) | Reached `-[NSApplication run]`; reused the existing state store, `meta.db` unchanged |

The machine has two pre-existing keychain items under service `com.dbpro.app`
(`agent/groq_api_key` and a saved-connection password), both confirmed present with
`security find-generic-password` (**metadata only — no secret was read**). So this host is a
valid reproducer for the stall condition. `KEYRING_SERVICE = "com.dbpro.app"` and
`KEYRING_GROQ_KEY = "agent/groq_api_key"` (`crates/native-app/src/main.rs:198,200`).

## 2. The captured stack — what it is actually blocked on

`sample 22096 6` — all **4762 of 4762 samples** (6 s at 1 ms) are in this single main-thread
stack. Excerpt, verbatim from `16-launch-a-keyring-stall.txt`:

```
4762 Thread_135352045   DispatchQueue_1: com.apple.main-thread  (serial)
+ 4762 start  (in dyld) + 6992
+   4762 main  (in db-pro-native) + 52
+     4762 std::rt::lang_start_internal
+       4762 db_pro_native::main::h1d723d9279319963  (in db-pro-native) + 732
+         4762 keyring::Entry::get_password::hc1622f049522d8c6  (in db-pro-native) + 168
+           4762 <keyring::macos::MacCredential as keyring::credential::CredentialApi>::get_password
+             4762 security_framework::os::macos::passwords::find_generic_password
+               4762 SecKeychainFindGenericPassword  (in Security) + 384
+                 4762 Security::KeychainCore::ItemImpl::getData
+                   4762 Security::KeychainCore::ItemImpl::getContent
+                     4762 Security::CssmClient::SSDbUniqueRecordImpl::get
+                       4762 Security::CssmClient::SSGroupImpl::decodeDataBlob
+                         4762 CSSM_DecryptDataFinal  (in Security)
+                           4762 Security::CSPFullPluginSession::CSPContext::final
+                             4762 SSCryptContext::outputSize
+                               4762 Security::SecurityServer::ClientSession::decrypt
+                                 4762 mach_msg  (in libsystem_kernel.dylib)
+                                   4762 mach_msg_overwrite
```

The call site is exactly the predicted one: `seed_groq_api_key_from_keyring()`
(`crates/native-app/src/main.rs:205-220`), which the crate's `main()` invokes **before**
`resolve_data_dir()`. `keyring::Entry::get_password()` → `SecKeychainFindGenericPassword` →
`CSSM_DecryptDataFinal` → `SecurityServer::ClientSession::decrypt` → `mach_msg`: the process is
parked in a Mach message waiting for **`securityd` to decrypt the item for it**, which requires
an interactive authorization grant for a binary that has no ACL entry on that item.

Corroborating side effects while stalled: `meta.db` inode/size/mtime **unchanged**, `/.db-pro-data`
**absent**, and **zero bytes** on stdout/stderr in 12 s. Startup never reached the data-directory
code. The block is unbounded — there is no timeout on this call.

## 3. Classification

- **(b) environment-specific — the decisive factor. YES.** The hang needs two conditions
  together: (i) a pre-existing `com.dbpro.app` keychain item, and (ii) a context where the
  authorization prompt cannot be presented. Launch B is the control that isolates (i): with no
  item reachable, the same binary passed the keyring instantly and started normally. Condition
  (ii) is this shell (no window-server/TCC access); on a normal desktop session macOS presents
  the authorization dialog and a user can answer it.
- **(c) keyring-backend behaviour — the mechanism. YES.** The block is inside Apple's Security
  framework waiting on `securityd`, not in DB Pro code. It is the documented consequence of an
  ad-hoc-signed binary (`R003`) that cannot match an existing keychain item's ACL.
- **(a) a real application bug — NO, but with a real design weakness.** There is no logic error:
  the code reads the keyring exactly as intended. The weakness is that the read is **unbounded
  and on the startup path before the state directory is resolved**, so the observable failure
  mode is "launch never completes" rather than "launch continues without the key".

## 4. Can it hang on a normal first launch? — the boundary condition

**No, not for a brand-new user.** A fresh install has no `com.dbpro.app` keychain item, so
`get_password()` returns item-not-found and returns promptly — empirically demonstrated by
Launch B. The stall requires the app to have stored a Groq key or a connection password
*previously*, and then to be launched in a context where the authorization prompt cannot be
shown (unattended/headless, or before the user notices the dialog).

So the practical exposure is: **a returning user, or any unattended launch, on a machine where
the app has already stored a keychain item.**

## 5. Severity and disposition

**Classified `P2` — recorded, not chased.** Reasoning against the policy in brief §40:

- Not `P0`: no data loss, no security leak, no destructive operation.
- Not `P1`: the app is not broken for a first launch (proven above), and in a GUI session this is
  an answerable one-time authorization prompt rather than a hard failure. It is also **already a
  known, disclosed item** (`R-KEYRING-STALL` / `LIM-018`), whose recorded owner decision is to
  keep it as a documented limitation for 0.1.0 because a fix is code work outside the closure
  pass. Nothing in this run changes that disposition, and this run is not authorised to take on
  new production work.
- It is therefore `P2` — a startup-path inconvenience with a disclosed workaround
  (answer the prompt once, or set `GROQ_API_KEY` to bypass the keyring read).

**No code was changed.** No placeholder key was used to paper over the stall: Launch A is the
unmodified reproduction, and Launch C is explicitly labelled a **bypass** everywhere it appears.

**Escalation condition for the coordinator:** if v0.1 is ever deployed to an unattended or
headless context, or if the prompt is found to recur on every launch after "Always Allow" (which
ad-hoc signing makes plausible, since the code identity changes across builds), this item should
be re-classified `P1` and fixed by making the keyring read bounded and off the pre-data-dir
critical path. That is a decision for the owner, not a change made here.

## 6. What was NOT verified

- The authorization prompt was never *seen* — no window can be displayed from this shell
  (`21-gui-unavailability-probes.txt`). The prompt's existence is inferred from the stack
  (`securityd` decrypt wait) plus macOS keychain semantics, not observed.
- Whether clicking "Always Allow" persists across app updates was not tested (that needs a GUI
  session and two builds).
- No live Agent/provider call was attempted; the stored key was never read or validated.
